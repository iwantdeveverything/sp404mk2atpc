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

#[tauri::command]
fn set_bus_effect(bus: String, slot: usize, effect: String, state: State<'_, AudioState>) -> Result<(), String> {
    let bus_enum = match bus.as_str() {
        "Bus1" => audio::state::BusRouting::Bus1,
        "Bus2" => audio::state::BusRouting::Bus2,
        "Dry" => audio::state::BusRouting::Dry,
        _ => return Err("Invalid bus".to_string()),
    };
    let effect_enum = match effect.as_str() {
        "Isolator" => audio::effects::EffectType::Isolator,
        "DjfxLooper" => audio::effects::EffectType::DjfxLooper,
        "VinylSim" => audio::effects::EffectType::VinylSim,
        "Filter" => audio::effects::EffectType::Filter,
        "Delay" => audio::effects::EffectType::Delay,
        "Reverb" => audio::effects::EffectType::Reverb,
        "Scatter" => audio::effects::EffectType::Scatter,
        "Slicer" => audio::effects::EffectType::Slicer,
        _ => return Err("Invalid effect".to_string()),
    };
    state.set_bus_effect(bus_enum, slot, effect_enum);
    Ok(())
}

#[tauri::command]
fn set_effect_param(bus: String, slot: usize, param_id: u8, value: f32, state: State<'_, AudioState>) -> Result<(), String> {
    let bus_enum = match bus.as_str() {
        "Bus1" => audio::state::BusRouting::Bus1,
        "Bus2" => audio::state::BusRouting::Bus2,
        "Dry" => audio::state::BusRouting::Dry,
        _ => return Err("Invalid bus".to_string()),
    };
    state.set_effect_param(bus_enum, slot, param_id, value);
    Ok(())
}

#[tauri::command]
fn remove_bus_effect(bus: String, slot: usize, state: State<'_, AudioState>) -> Result<(), String> {
    let bus_enum = match bus.as_str() {
        "Bus1" => audio::state::BusRouting::Bus1,
        "Bus2" => audio::state::BusRouting::Bus2,
        "Dry" => audio::state::BusRouting::Dry,
        _ => return Err("Invalid bus".to_string()),
    };
    state.remove_bus_effect(bus_enum, slot);
    Ok(())
}

#[tauri::command]
fn set_tempo(bpm: f32, state: State<'_, AudioState>) -> Result<(), String> {
    state.set_tempo(bpm);
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
        .invoke_handler(tauri::generate_handler![load_audio, trigger_pad, set_resampling, set_pad_bus, set_bus_effect, set_effect_param, remove_bus_effect, set_tempo])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
