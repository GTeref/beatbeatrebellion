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
    let mut beatmap=Vec::new();
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
            let threshold=prev*1.5+0.1; //no idea what im doing here, adaptive?

            if curr>threshold && energy_increase>0.05 {
                let timestamp=(chunk_start as f32) / sample_rate as f32;

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
        prev_energies = band_energies

    }
    let processed_beatmap=post_process_beatmap(beatmap);
    Ok(processed_beatmap)

}

fn determine_note_type(energies: &[f32], band_idx: usize, samples: &[f32], start_pos:usize, sample_rate:usize, hsize:usize ) -> (String, f32) {
    let curr_energy=energies[band_idx];

    let lookahead_chunks=10; //look ahead for 0.n secs
    let mut sustained_energy=0;

    for i in 1..=lookahead_chunks {
        let future_pos=start_pos + (i * hsize);
        if future_pos + 512<samples.len() { // 512 is chunksize, need to make it global later
            let future_chunk=&samples[future_pos..future_pos + 512];
            let avg_energy:f32= future_chunk.iter()
                .map(|x| x.abs()) // absolute value for energy
                .sum::<f32>() / future_chunk.len() as f32;

            if avg_energy > curr_energy * 0.7 { 
                sustained_energy += 1;
            }
        }
    }

    //if energy is sustained for more than half of lookahead period, it's a hold note
    if sustained_energy>=lookahead_chunks/2 {
        let duration = (lookahead_chunks * hsize) as f32 / sample_rate as f32; // duration in seconds
        return ("hold".to_string(), duration); // add min hold duration?
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

        while cluster_end + 1 < beatmap.len() {
            let next=&beatmap[cluster_end + 1];
            if next.lane == curr.lane && (next.timestamp - curr.timestamp) < 0.15 {
                cluster_end += 1;
            } else {
                break;
            }
        }

        // found cluster,convert to hold note
        if cluster_end>i{
            let duration=beatmap[cluster_end].timestamp - curr.timestamp + 0.1;
            processed.push(BeatmapNote {
                timestamp: curr.timestamp,
                lane: curr.lane,
                note_type: "hold".to_string(),
                duration,
                energy: curr.energy, // average energy of the cluster
            });
        } else {
            processed.push(BeatmapNote {
                timestamp: curr.timestamp,
                lane: curr.lane,
                note_type: "single".to_string(),
                duration: 0.0, // single notes have no duration
                energy: curr.energy,
            });
        }

        i= cluster_end + 1;
    }
    processed

}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![analyze_audio])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
