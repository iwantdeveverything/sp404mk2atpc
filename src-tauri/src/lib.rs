pub mod audio;
pub mod fs;

use audio::state::AudioState;
use audio::effects::{effect_metadata, effect_type_from_str, implemented_effects, Curve};
use std::path::Path;
use tauri::State;

/// Serializable form of `ParamSpec` for the IPC boundary. `ParamSpec` itself
/// holds `&'static str`, which serializes fine, but the DTO gives the frontend a
/// stable, owned shape and a string curve.
#[derive(serde::Serialize)]
struct ParamSpecDto {
    name: String,
    unit: String,
    min: f32,
    max: f32,
    default: f32,
    curve: String,
}

fn curve_label(curve: Curve) -> &'static str {
    match curve {
        Curve::Linear => "linear",
        Curve::Exponential => "exponential",
    }
}

fn effect_label(effect: audio::effects::EffectType) -> &'static str {
    use audio::effects::EffectType;
    match effect {
        EffectType::Filter => "Filter",
        EffectType::Isolator => "Isolator",
        EffectType::Delay => "Delay",
        EffectType::Reverb => "Reverb",
        EffectType::VinylSim => "VinylSim",
        EffectType::DjfxLooper => "DjfxLooper",
        EffectType::Scatter => "Scatter",
        EffectType::Slicer => "Slicer",
        // PR2: Modulation family
        EffectType::Tremolo => "Tremolo",
        EffectType::AutoPan => "AutoPan",
        EffectType::Chorus => "Chorus",
        EffectType::Flanger => "Flanger",
        EffectType::Phaser => "Phaser",
        _ => "",
    }
}

#[tauri::command]
fn load_audio(path: String, pad_id: usize, state: State<'_, AudioState>) -> Result<(), String> {
    let audio_buffer = fs::audio::load_file(Path::new(&path))?;
    state.add_buffer(pad_id, audio_buffer);
    Ok(())
}

#[tauri::command]
fn pre_listen_start(path: String, state: State<'_, AudioState>) -> Result<(), String> {
    let audio_buffer = fs::audio::load_file(Path::new(&path))?;
    state.pre_listen(audio_buffer);
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
    let effect_enum = effect_type_from_str(&effect).ok_or("Invalid effect")?;
    state.set_bus_effect(bus_enum, slot, effect_enum);
    Ok(())
}

/// List the implemented effects (single source of truth for the selector).
/// Pure, off the audio thread, never enqueues a command.
#[tauri::command]
fn list_effects() -> Vec<String> {
    implemented_effects()
        .iter()
        .map(|&e| effect_label(e).to_string())
        .collect()
}

/// Return the parameter metadata for an effect so the UI can render N controls.
/// Pure, off the audio thread, never enqueues a command.
#[tauri::command]
fn get_effect_parameters(effect: String) -> Result<Vec<ParamSpecDto>, String> {
    let effect_enum = effect_type_from_str(&effect).ok_or("Invalid effect")?;
    let dtos = effect_metadata(effect_enum)
        .iter()
        .map(|spec| ParamSpecDto {
            name: spec.name.to_string(),
            unit: spec.unit.to_string(),
            min: spec.min,
            max: spec.max,
            default: spec.default,
            curve: curve_label(spec.curve).to_string(),
        })
        .collect();
    Ok(dtos)
}

#[tauri::command]
fn set_effect_mix(bus: String, slot: usize, mix: f32, state: State<'_, AudioState>) -> Result<(), String> {
    let bus_enum = match bus.as_str() {
        "Bus1" => audio::state::BusRouting::Bus1,
        "Bus2" => audio::state::BusRouting::Bus2,
        "Dry" => audio::state::BusRouting::Dry,
        _ => return Err("Invalid bus".to_string()),
    };
    state.set_effect_mix(bus_enum, slot, mix);
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
    
    let auto_save_engine = fs::project::AutoSaveEngine::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(audio_state)
        .manage(auto_save_engine)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_audio, 
            trigger_pad, 
            set_resampling, 
            set_pad_bus, 
            set_bus_effect,
            set_effect_param,
            set_effect_mix,
            remove_bus_effect,
            list_effects,
            get_effect_parameters,
            set_tempo,
            pre_listen_start,
            fs::project::list_directory,
            fs::project::ingest_sample_to_project,
            fs::project::save_project_state,
            fs::project::get_default_project_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
