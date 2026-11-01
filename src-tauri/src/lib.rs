// Register the existing backend modules without changing their layout.
mod agent;
mod cargo_exec;
mod config;
mod db;
mod error;
mod fs;

use fs::explorer::{
    FileEntry, ProjectState, canonical_project_root, list_directory, read_project_file,
};
use std::path::Path;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn open_project(
    app: tauri::AppHandle,
    project: State<'_, ProjectState>,
) -> Result<Option<String>, String> {
    let selected =
        tauri::async_runtime::spawn_blocking(move || app.dialog().file().blocking_pick_folder())
            .await
            .map_err(|error| format!("folder dialog task failed: {error}"))?;

    let Some(selected) = selected else {
        return Ok(None);
    };

    let selected = selected
        .into_path()
        .map_err(|error| format!("invalid selected path: {error}"))?;

    let root = canonical_project_root(&selected)
        .map_err(|error| format!("could not open project: {error}"))?;

    let mut current = project
        .root
        .lock()
        .map_err(|_| "project state lock is unavailable".to_string())?;
    *current = Some(root.clone());

    Ok(Some(root.to_string_lossy().into_owned()))
}

#[tauri::command]
async fn list_project_directory(
    project: State<'_, ProjectState>,
    relative_path: Option<String>,
) -> Result<Vec<FileEntry>, String> {
    let root = {
        let current = project
            .root
            .lock()
            .map_err(|_| "project state lock is unavailable".to_string())?;
        current
            .clone()
            .ok_or_else(|| "open a project before listing files".to_string())?
    };

    let relative_path = relative_path.unwrap_or_default();

    tauri::async_runtime::spawn_blocking(move || {
        list_directory(&root, Path::new(&relative_path))
            .map_err(|error| format!("could not list directory: {error}"))
    })
    .await
    .map_err(|error| format!("directory task failed: {error}"))?
}

// Read the contents of a selected project file.
//
// The frontend supplies only a project-relative path.
// The native backend retrieves the authorized root.
#[tauri::command]
async fn read_project_source(
    project: State<'_, ProjectState>,
    relative_path: String,
) -> Result<String, String> {
    // Step 1: Read the currently selected project root.
    let root = {
        // Look the state shared by backend commands.
        let current = project
            .root
            .lock()
            .map_err(|_| "project state lock is unavailable".to_string())?;

        // Clone the path so we can release the mutex.
        // before performing filesystem operations.
        //
        // If no preject is selected, return an error.
        current
            .clone()
            .ok_or_else(|| "open a project before reading files.".to_string())?
    };

    // Step 2: Move blocking filesystem work to a worker.
    //
    // `move` transfers ownerships of root and relative path.
    // into this closure.
    tauri::async_runtime::spawn_blocking(move || {
        // Step 3: Call the filesystem function we just wrote.
        read_project_file(&root, Path::new(&relative_path))
            .map_err(|error| format!("could not read file: {error}"))
    })
    .await
    .map_err(|error| format!("file read task failed: {error}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ProjectState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            open_project,
            list_project_directory,
            read_project_source,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
