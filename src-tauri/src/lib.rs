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
    state.trigger_pad(pad_id, None);
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
        .invoke_handler(tauri::generate_handler![load_audio, trigger_pad])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
