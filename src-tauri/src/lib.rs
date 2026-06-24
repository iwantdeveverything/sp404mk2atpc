pub mod audio;
pub mod fs;

use audio::state::AudioState;
use std::path::Path;
use tauri::State;

#[tauri::command]
fn load_audio(path: String, pad_id: usize, state: State<'_, AudioState>) -> Result<(), String> {
    let audio_buffer = fs::audio::load_file(Path::new(&path))?;
    state.add_buffer(pad_id, audio_buffer);
    Ok(())
}

#[tauri::command]
fn trigger_pad(pad_id: usize, state: State<'_, AudioState>) -> Result<(), String> {
    state.trigger_pad(pad_id, None, audio::state::BusRouting::Dry);
    Ok(())
}

#[tauri::command]
fn set_resampling(state: bool) -> Result<(), String> {
    println!("Resampling state set to: {}", state);
    Ok(())
}

#[tauri::command]
fn set_pad_bus(pad: usize, bus: String) -> Result<(), String> {
    println!("Pad {} routed to {}", pad, bus);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (audio_state, consumer) = AudioState::new(1024);

    let stream = audio::engine::start_audio_engine(audio_state.clone(), consumer)
        .expect("Failed to start audio engine");

    // Leak the stream to keep it alive for the lifetime of the application
    Box::leak(Box::new(stream));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(audio_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_audio, trigger_pad, set_resampling, set_pad_bus])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
