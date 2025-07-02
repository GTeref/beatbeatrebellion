// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use rodio::{Decoder, Source};
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use std::io::Cursor;
use serde::{Deserialize, Serialize};

// #[tauri::command]
// fn greet(name: &str) -> String {
//     format!("Hello, {}! You've been greeted from Rust!", name)
// }

#[derive(Serialize, Deserialize)]
struct BeatmapNote{
    timestamp: f32, // in seconds
    lane: usize, // 0-3 for 4 lanes
    note_type: String, // "single" or "hold"
    duration: f32,
    energy: f32
}

#[tauri::command]
fn analyze_audio(audio_data: Vec<u8>)->Result<Vec<BeatmapNote>, String> {
    //convert audio bytes to samples
    let cursor=Cursor::new(audio_data);
    let source=match Decoder::new(cursor){
        Ok(decoder)=>decoder,
        Err(e)=>return Err(format!("Error decoding audio: {}", e)),
    };

    let sample_rate=source.sample_rate();
    let channels=source.channels() as usize;

    let samples:Vec<f32>=source.convert_samples::<f32>().collect();

    // chunks for analysis
    let csize=512; //chunksize
    let hsize=256; //hopsize
    let mut beatmap: Vec<BeatmapNote>=Vec::new();
    let mut prev_energies=vec![0.0; 4]; // onset detection

    for (chunk_idx,chunk_start) in (0..samples.len()).step_by(hsize).enumerate() {

        let chunk_end=(chunk_start + csize).min(samples.len());
        if (chunk_end-chunk_start < csize) { break; }

        let chunk=&samples[chunk_start..chunk_end];

        let mono_chunk:Vec<f32>=if channels == 2{
            chunk.chunks_exact(2)
                .map(|stereo| (stereo[0]+stereo[1])/2.0)
                .collect()
        } else{
            chunk.to_vec()
        };
            
        //FFT
        let hann_window=spectrum_analyzer::windows::hann_window(&mono_chunk);
        let spectrum=samples_fft_to_spectrum(
            &hann_window,
            sample_rate as u32,
            FrequencyLimit::Range(20.0,20000.0),
            None
        ).map_err(|e| format!("FFT error: {}", e))?;

        let bands=[
            (20.0, 100.0), //bass
            (100.0, 300.0), //low-mid
            (300.0, 800.0), //mid
            (800.0, 2000.0) //high
        ];

        let mut band_energies=Vec::new();
        for (low,high) in bands.iter(){
            let energy: f32=spectrum.data().iter()
                .filter(|(freq,_)| {
                    let freq_val = freq.val();
                    freq_val >= *low && freq_val < *high
                })
                .map(|(_,magnitude)| magnitude.val())
                .sum();
            band_energies.push(energy);
        }

        //onset detection
        for (band_idx,(&curr,&prev)) in band_energies.iter().zip(prev_energies.iter()).enumerate() {
            let energy_increase= curr - prev;

            let adaptive_threshold=prev*2.0+0.2; // threshold for energy increase
            let min_energy_increase=0.15;
            let min_energy=0.3;

            if curr>adaptive_threshold && energy_increase>min_energy_increase && curr>min_energy {
                let timestamp=(chunk_start as f32) / sample_rate as f32;

                //minimum time gap between notes in same lane
                let min_gap=0.1;

                if let Some(last_note) = beatmap.iter().rev().find(|n| n.lane == band_idx) {
                    if (timestamp - last_note.timestamp) < min_gap {
                        continue; // skip if too close to last note
                    }
                }

                let (note_type, duration) = determine_note_type(&band_energies, band_idx, &samples, chunk_start, sample_rate as usize, hsize);

                beatmap.push(BeatmapNote {
                    timestamp,
                    lane: band_idx,
                    note_type,
                    duration,
                    energy: curr, // store current energy for the note
                });
            }
        }
        prev_energies = band_energies;

    }
    let processed_beatmap=post_process_beatmap(beatmap);
    let balanced_beatmap=balance_lanes(processed_beatmap);
    Ok(balanced_beatmap)

}

fn determine_note_type(energies: &[f32], band_idx: usize, samples: &[f32], start_pos:usize, sample_rate:usize, hsize:usize ) -> (String, f32) {
    let curr_energy=energies[band_idx];

    let lookahead_chunks=8; //look ahead for 0.n secs
    let mut sustained_energy=0;
    let min_hold_threshold=curr_energy*0.6;

    for i in 1..=lookahead_chunks {
        let future_pos=start_pos + (i * hsize);
        if future_pos + 512<samples.len() { // 512 is chunksize, need to make it global later
            let future_chunk=&samples[future_pos..future_pos + 512];
            let avg_energy:f32= future_chunk.iter()
                .map(|x| x.abs()) // absolute value for energy
                .sum::<f32>() / future_chunk.len() as f32;

            if avg_energy > min_hold_threshold{ 
                sustained_energy += 1;
            } else {
                break;
            }
        }
    }

    //if energy is sustained for more than half of lookahead period, it's a hold note
    if sustained_energy >= lookahead_chunks / 2 {
        let duration = (sustained_energy as f32 * hsize as f32) / sample_rate as f32; // duration in seconds
        return ("hold".to_string(), duration.max(0.3).min(1.5)); // add min hold duration?
    } else {
        return ("single".to_string(), 0.0); // single note, no duration
    }
}

fn post_process_beatmap(mut beatmap: Vec<BeatmapNote>) -> Vec<BeatmapNote> {
    beatmap.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

    let mut processed= Vec::new();
    let mut i=0;

    while i<beatmap.len(){
        let curr=&beatmap[i];
        let mut cluster_end=i;

        let cluster_window=0.12;

        while cluster_end + 1 < beatmap.len() {
            let next=&beatmap[cluster_end + 1];
            if next.lane == curr.lane && (next.timestamp - curr.timestamp) < cluster_window {
                cluster_end += 1;
            } else {
                break;
            }
        }

        // found cluster of 2, convert to hold note
        if cluster_end>i {
            let duration=beatmap[cluster_end].timestamp - curr.timestamp + 0.1;
            processed.push(BeatmapNote {
                timestamp: curr.timestamp,
                lane: curr.lane,
                note_type: "hold".to_string(),
                duration: duration.min(3.0),
                energy: curr.energy, // average energy of the cluster
            });
        } else {
            for j in i..=cluster_end {
                let curr = &beatmap[j];
                processed.push(BeatmapNote {
                    timestamp: curr.timestamp,
                    lane: curr.lane,
                    note_type: "single".to_string(),
                    duration: 0.0, // single notes have no duration
                    energy: curr.energy,
                });
            }
        }

        i= cluster_end + 1;
    }
    // remove notes too close to each other
    let mut final_processed : Vec<BeatmapNote>= Vec::new();
    let min_global_gap=0.03;

    for note in processed {
        if let Some(last_note)=final_processed.last() {
            if note.timestamp - last_note.timestamp < min_global_gap {
                continue; // skip if too close to last note
            }
        }
        final_processed.push(note);
    }
    final_processed

}

fn balance_lanes(mut beatmap: Vec<BeatmapNote>)->Vec<BeatmapNote> {
    // Balance notes across lanes
    let mut lane_counts = vec![0; 4]; // 4 lanes
    for note in &beatmap {
        lane_counts[note.lane] += 1;
    }

    let max_notes= *lane_counts.iter().max().unwrap_or(&0);
    let target_notes = max_notes / 2; // target is to have equal notes in each lane

    let mut idx_to_remove=Vec::new();

    for lane in 0..4 {
        if lane_counts[lane] > target_notes{
            let mut lane_notes: Vec<(usize,&BeatmapNote)> = beatmap.iter()
                .enumerate()
                .filter(|(_,n)| n.lane == lane)
                .collect();
            
            // sort by energy, highest first
            lane_notes.sort_by(|a, b| b.1.energy.partial_cmp(&a.1.energy).unwrap());

            let to_remove = lane_counts[lane] - target_notes;
            for (original_idx, _) in lane_notes.iter().skip(target_notes).take(to_remove) {
                idx_to_remove.push(*original_idx);
            }
        }
    }

    for idx in idx_to_remove {
        beatmap[idx].energy = -1.0; // mark for removal
    }

    beatmap.retain(|note| note.energy >= 0.0); // remove marked notes

    beatmap
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![analyze_audio])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
