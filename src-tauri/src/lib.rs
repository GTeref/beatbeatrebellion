// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn analyze_audio(audio_data: Vec<u8>)->Result<Vec<Vec<f32>>, String> {
    //convert audio bytes to samples
    let cursor=Cursor::new(audio_data);
    let source=match Decoder::new(cursor){
        Ok(decoder)=>decoder,
        Err(e)=>return Err(format!("Error decoding audio: {}", e)),
    }

    let sample_rate=source.sample_rate();
    let channels=source.channels() as usize

    let samples:Vec<f32>=source.convert_samples::<f32>().collect()

    // chunks for analysis
    let csize=1024
    let mut beatmap=Vec::new()

    for chunk in samples.chunks(csize*channels){
        let mono_chunk:Vec<f32>=if channels == 2{
            chunk.chunks_exact(2)
                .map(|stereo| (stereo[0]+stereo[1])/2.0)
                .collect()
        } else{
            chunk.to_vec()
            
        //FFT
        let hann_window=spectrum_analyzer::windows::hann_window(&mono_chunk)
        let spectrum=samples_fft_to_spectrum(
            &hann_window,
            sample_rate as u32,
            FrequencyLimit::Range(20.0,20000.0),
            None
        ).map_err(|e| format!("FFT error: {}", e))?

        let bands=[
            (20.0, 100.0), //bass
            (100.0, 300.0), //low-mid
            (300.0, 800.0), //mid
            (800.0, 2000.0) //high
        ]

        let mut band_energies=Vec::new()
        for (low,high) in bands.iter(){
            let energy: f32=spectrum.iter()
                .filter(|(freq,_)| freq >= low && freq <= high)
                .map(|(_,magnitude)| magnitude.val())
                .sum()
            band_energies.push(energy)
        }

        beatmap.push(band_energies)
    }
    Ok(beatmap)

}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
