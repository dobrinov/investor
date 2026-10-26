//! Investor as a desktop app: the same web app in a native window, with
//! native dialogs to open and save `.invest` files in place. The app's files
//! ship inside the bundle. The only network traffic is to the AI model the
//! person sets up in the assistant, when they use it, and the check for a
//! newer version of the app on GitHub.

// No console window next to the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bridge;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use tauri::ipc::{InvokeBody, Request, Response};
use tauri::{AppHandle, Manager, RunEvent, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_updater::{Update, UpdaterExt};

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

/// A request to the AI model the person set up: a local server such as
/// Ollama or LM Studio, or a provider's API with their own key. Made here
/// rather than in the web view, which may only talk to the app itself.
#[derive(Deserialize)]
struct ModelRequest {
    method: String,
    url: String,
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(default)]
    body: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct ModelReply {
    status: u16,
    body: String,
}

#[tauri::command]
async fn model_request(request: ModelRequest) -> Result<ModelReply, String> {
    if !(request.url.starts_with("http://") || request.url.starts_with("https://")) {
        return Err("the model's address must start with http:// or https://".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        // Errors come back as a status and body for the app to explain.
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(600)))
            .build()
            .into();
        let mut response = match (request.method.as_str(), request.body) {
            ("GET", _) => {
                let mut call = agent.get(&request.url);
                for (k, v) in &request.headers {
                    call = call.header(k, v);
                }
                call.call()
            }
            (_, body) => {
                let mut call = agent.post(&request.url);
                for (k, v) in &request.headers {
                    call = call.header(k, v);
                }
                call.send_json(body.unwrap_or(serde_json::Value::Null))
            }
        }
        .map_err(|e| e.to_string())?;
        let status = response.status().as_u16();
        let body = response.body_mut().with_config().limit(32 * 1024 * 1024).read_to_string().map_err(|e| e.to_string())?;
        Ok(ModelReply { status, body })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// When the file was last changed, in milliseconds since 1970.
#[tauri::command]
async fn modified_invest(path: String) -> Result<u64, String> {
    let modified = std::fs::metadata(&path).and_then(|m| m.modified()).map_err(|e| e.to_string())?;
    Ok(modified.duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis() as u64)
}

/// A newer version found by `update_check`, waiting to be installed.
#[derive(Default)]
struct Pending(Mutex<Option<Update>>);

#[derive(Serialize)]
struct Available {
    version: String,
    notes: Option<String>,
}

/// Looks for a newer version of the app among the GitHub releases.
#[tauri::command]
async fn update_check(app: AppHandle, pending: State<'_, Pending>) -> Result<Option<Available>, String> {
    let update = app.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| e.to_string())?;
    let available = update.as_ref().map(|u| Available { version: u.version.clone(), notes: u.body.clone() });
    *pending.0.lock().map_err(|e| e.to_string())? = update;
    Ok(available)
}

/// Downloads and installs the version `update_check` found, then restarts.
#[tauri::command]
async fn update_install(app: AppHandle, pending: State<'_, Pending>) -> Result<(), String> {
    let update = pending.0.lock().map_err(|e| e.to_string())?.take().ok_or("no update is waiting")?;
    update.download_and_install(|_, _| {}, || {}).await.map_err(|e| e.to_string())?;
    app.restart()
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Launch(Mutex::new(from_args)))
        .manage(Pending::default())
        .manage(bridge::Bridge::default())
        .invoke_handler(tauri::generate_handler![
            pick_invest,
            save_invest_as,
            read_invest,
            write_invest,
            launch_file,
            model_request,
            modified_invest,
            update_check,
            update_install,
            bridge::bridge_start,
            bridge::bridge_stop,
            bridge::bridge_next,
            bridge::bridge_reply,
        ])
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
