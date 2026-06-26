use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;
use tauri::{Manager, State};

#[derive(Serialize)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[tauri::command]
pub fn list_directory(path: String) -> Result<Vec<DirEntry>, String> {
    let p = Path::new(&path);
    if !p.is_dir() {
        return Err("Not a directory".into());
    }

    let mut entries = Vec::new();
    if let Ok(read_dir) = fs::read_dir(p) {
        for entry in read_dir.flatten() {
            if let Ok(file_type) = entry.file_type() {
                entries.push(DirEntry {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: entry.path().to_string_lossy().to_string(),
                    is_dir: file_type.is_dir(),
                });
            }
        }
    } else {
        return Err("Failed to read directory".into());
    }
    
    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name))
    });

    Ok(entries)
}

#[tauri::command]
pub fn ingest_sample_to_project(source_path: String, project_dir: String) -> Result<String, String> {
    let source = Path::new(&source_path);
    if !source.is_file() {
        return Err("Source is not a valid file".into());
    }
    
    let samples_dir = Path::new(&project_dir).join("samples");
    if !samples_dir.exists() {
        fs::create_dir_all(&samples_dir).map_err(|e| e.to_string())?;
    }
    
    let file_name = source.file_name().ok_or("Invalid file name")?;
    let dest_path = samples_dir.join(file_name);
    
    fs::copy(source, &dest_path).map_err(|e| e.to_string())?;
    
    let relative_path = Path::new("samples").join(file_name);
    Ok(relative_path.to_string_lossy().to_string())
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectState {
    pub project_dir: String,
    pub data: serde_json::Value,
}

pub struct AutoSaveEngine {
    tx: Sender<ProjectState>,
}

impl AutoSaveEngine {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<ProjectState>();
        
        thread::spawn(move || {
            let mut last_state: Option<ProjectState> = None;
            loop {
                match rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(state) => {
                        last_state = Some(state);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if let Some(state) = last_state.take() {
                            let p = Path::new(&state.project_dir).join("project.json");
                            if let Ok(json) = serde_json::to_string_pretty(&state.data) {
                                let _ = fs::write(p, json);
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }
        });
        
        Self { tx }
    }

    pub fn save(&self, state: ProjectState) -> Result<(), String> {
        self.tx.send(state).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn save_project_state(state: ProjectState, engine: State<'_, AutoSaveEngine>) -> Result<(), String> {
    engine.save(state)
}

/// Pure helper: resolve the default project directory under `base`,
/// creating the `default-project` subfolder if it does not exist.
/// Kept free of any Tauri AppHandle dependency so it is unit-testable.
pub fn resolve_default_project_dir(base: &Path) -> Result<PathBuf, String> {
    let dir = base.join("default-project");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Resolve a stable default project directory under the app data dir.
/// The frontend has no notion of a project path; Rust owns it.
#[tauri::command]
pub fn get_default_project_dir(app: tauri::AppHandle) -> Result<String, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let dir = resolve_default_project_dir(&base)?;
    Ok(dir.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_resolve_default_project_dir_creates_dir() {
        // Unique base under the OS temp dir to avoid cross-test collisions.
        let mut base = temp_dir();
        base.push(format!("sp404_test_{}", std::process::id()));

        let resolved = resolve_default_project_dir(&base).expect("should resolve dir");

        assert!(resolved.is_dir(), "default project dir should be created");
        assert!(
            resolved.ends_with("default-project"),
            "resolved path should end with 'default-project'"
        );

        // Clean up.
        let _ = fs::remove_dir_all(&base);
    }
}
