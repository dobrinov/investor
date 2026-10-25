//! Investor as a desktop app: the same web app in a native window, with
//! native dialogs to open and save `.invest` files in place. Nothing here
//! talks to the network; the app's files ship inside the bundle.

// No console window next to the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use percent_encoding::percent_decode_str;
use serde::Serialize;
use tauri::ipc::{InvokeBody, Request, Response};
use tauri::{AppHandle, Manager, RunEvent, State};
use tauri_plugin_dialog::DialogExt;

/// A `.invest` file the app was asked to open when it started.
#[derive(Default)]
struct Launch(Mutex<Option<PathBuf>>);

#[derive(Serialize)]
struct Picked {
    path: String,
    name: String,
}

fn picked(path: &Path) -> Picked {
    Picked {
        path: path.to_string_lossy().into_owned(),
        name: path.file_name().map_or_else(|| "portfolio.invest".into(), |n| n.to_string_lossy().into_owned()),
    }
}

fn is_invest(path: &Path) -> bool {
    path.extension().is_some_and(|e| e.eq_ignore_ascii_case("invest"))
}

/// Asks which file to open.
#[tauri::command]
async fn pick_invest(app: AppHandle) -> Option<Picked> {
    let file = app.dialog().file().add_filter("Investor file", &["invest"]).blocking_pick_file()?;
    file.into_path().ok().map(|p| picked(&p))
}

/// Asks where to save, suggesting `name`.
#[tauri::command]
async fn save_invest_as(app: AppHandle, name: String) -> Option<Picked> {
    let file = app.dialog().file().add_filter("Investor file", &["invest"]).set_file_name(&name).blocking_save_file()?;
    let mut path = file.into_path().ok()?;
    if !is_invest(&path) {
        path.set_extension("invest");
    }
    Some(picked(&path))
}

#[tauri::command]
async fn read_invest(path: String) -> Result<Response, String> {
    std::fs::read(&path).map(Response::new).map_err(|e| e.to_string())
}

/// Writes the encrypted bytes (the request body) to the path in the `path`
/// header, by way of a temporary file so a failed save never leaves half a file.
#[tauri::command]
async fn write_invest(request: Request<'_>) -> Result<(), String> {
    let InvokeBody::Raw(bytes) = request.body() else { return Err("expected the file's bytes".into()) };
    let header = request.headers().get("path").and_then(|v| v.to_str().ok()).ok_or("no path given")?;
    let path = PathBuf::from(percent_decode_str(header).decode_utf8().map_err(|e| e.to_string())?.into_owned());
    if !is_invest(&path) {
        return Err("only .invest files are written".into());
    }
    let temporary = path.with_extension("invest.saving");
    std::fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&temporary, &path).map_err(|e| e.to_string())
}

/// The file the app was opened with, once.
#[tauri::command]
fn launch_file(launch: State<'_, Launch>) -> Option<Picked> {
    launch.0.lock().ok()?.take().map(|p| picked(&p))
}

fn main() {
    // Windows and Linux pass an opened file as an argument.
    let from_args = std::env::args_os().skip(1).map(PathBuf::from).find(|p| is_invest(p));
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Launch(Mutex::new(from_args)))
        .invoke_handler(tauri::generate_handler![pick_invest, save_invest_as, read_invest, write_invest, launch_file])
        .build(tauri::generate_context!())
        .expect("the app starts");
    app.run(|handle, event| {
        // macOS sends files opened from Finder as an event instead.
        #[cfg(target_os = "macos")]
        if let RunEvent::Opened { urls } = &event
            && let Some(path) = urls.iter().filter_map(|u| u.to_file_path().ok()).find(|p| is_invest(p))
            && let Ok(mut slot) = handle.state::<Launch>().0.lock()
        {
            *slot = Some(path);
        }
        let _ = (handle, event);
    });
}
