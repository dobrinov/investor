//! Lets AI tools on this computer (Claude Code, Cursor and other MCP clients)
//! use the file that is open in the app, over MCP's HTTP transport on
//! 127.0.0.1. The person turns it on in the app; requests need the address's
//! secret, and web pages are refused. Each request is handed to the web view,
//! which answers it with the unlocked file and asks the person before any
//! change, so the password and the file never leave the app.

use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use serde_json::{Value, json};
use tauri::State;
use tiny_http::{Header, Method, Request, Response, Server};

/// How long a request waits for the app, including the person's approval.
const ANSWER_WITHIN: Duration = Duration::from_secs(30 * 60);
const BODY_LIMIT: u64 = 4 * 1024 * 1024;

#[derive(Clone, Serialize)]
pub struct Job {
    id: u64,
    message: Value,
}

pub struct Bridge {
    jobs: Sender<Job>,
    inbox: Arc<Mutex<Receiver<Job>>>,
    waiting: Arc<Mutex<HashMap<u64, SyncSender<Value>>>>,
    next: Arc<AtomicU64>,
    server: Mutex<Option<Arc<Server>>>,
}

impl Default for Bridge {
    fn default() -> Self {
        let (jobs, inbox) = channel();
        Self { jobs, inbox: Arc::new(Mutex::new(inbox)), waiting: Arc::default(), next: Arc::default(), server: Mutex::default() }
    }
}

impl Bridge {
    fn stop(&self) {
        if let Some(server) = self.server.lock().ok().and_then(|mut s| s.take()) {
            server.unblock();
        }
    }
}

/// Starts answering on `127.0.0.1:port`, replacing an earlier start.
#[tauri::command]
pub fn bridge_start(bridge: State<'_, Bridge>, port: u16, secret: String) -> Result<(), String> {
    listen(&bridge, port, secret)
}

fn listen(bridge: &Bridge, port: u16, secret: String) -> Result<(), String> {
    bridge.stop();
    if secret.len() < 16 {
        return Err("the secret is too short".into());
    }
    let server = Arc::new(Server::http(("127.0.0.1", port)).map_err(|e| format!("port {port} is in use or not allowed: {e}"))?);
    *bridge.server.lock().map_err(|e| e.to_string())? = Some(server.clone());
    let (jobs, waiting, next) = (bridge.jobs.clone(), bridge.waiting.clone(), bridge.next.clone());
    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            let (jobs, waiting, next, secret) = (jobs.clone(), waiting.clone(), next.clone(), secret.clone());
            std::thread::spawn(move || answer(request, &secret, port, &jobs, &waiting, &next));
        }
    });
    Ok(())
}

#[tauri::command]
pub fn bridge_stop(bridge: State<'_, Bridge>) {
    bridge.stop();
}

/// The next request for the web view to answer, or nothing after a while.
#[tauri::command]
pub async fn bridge_next(bridge: State<'_, Bridge>) -> Result<Option<Job>, String> {
    let inbox = bridge.inbox.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let inbox = inbox.lock().map_err(|e| e.to_string())?;
        Ok(inbox.recv_timeout(Duration::from_secs(20)).ok())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The web view's answer to request `id`: a whole JSON-RPC reply.
#[tauri::command]
pub fn bridge_reply(bridge: State<'_, Bridge>, id: u64, reply: Value) {
    if let Some(sender) = bridge.waiting.lock().ok().and_then(|mut w| w.remove(&id)) {
        let _ = sender.send(reply);
    }
}

fn answer(mut request: Request, secret: &str, port: u16, jobs: &Sender<Job>, waiting: &Mutex<HashMap<u64, SyncSender<Value>>>, next: &AtomicU64) {
    let header = |name: &'static str| request.headers().iter().find(|h| h.field.equiv(name)).map(|h| h.value.as_str().to_owned());
    // Web pages may not reach the file: no browser origin, and only our own
    // host name, which also stops DNS rebinding.
    let host_ok = header("Host").is_some_and(|h| h == format!("127.0.0.1:{port}") || h == format!("localhost:{port}"));
    let from_page = header("Origin").is_some_and(|o| o != "null" && !o.is_empty());
    let path = request.url().split('?').next().unwrap_or_default().trim_end_matches('/').to_owned();
    let bearer = header("Authorization").and_then(|a| a.strip_prefix("Bearer ").map(str::to_owned));
    let secret_ok = path == format!("/mcp/{secret}") || (path == "/mcp" && bearer.as_deref() == Some(secret));
    let reply = |status: u16, body: String| {
        let mut response = Response::from_string(body).with_status_code(status);
        if let Ok(h) = Header::from_bytes("Content-Type", "application/json") {
            response.add_header(h);
        }
        response
    };
    if !host_ok || from_page {
        let _ = request.respond(reply(403, r#"{"error":"only AI tools on this computer may connect"}"#.into()));
        return;
    }
    if !secret_ok {
        let _ = request.respond(reply(401, r#"{"error":"copy the address from the Investor app again"}"#.into()));
        return;
    }
    if *request.method() != Method::Post {
        // No server-to-client stream: everything is a reply to a request.
        let _ = request.respond(Response::empty(405).with_header(Header::from_bytes("Allow", "POST").expect("a valid header")));
        return;
    }
    let mut body = String::new();
    if request.as_reader().take(BODY_LIMIT).read_to_string(&mut body).is_err() {
        let _ = request.respond(reply(400, r#"{"error":"unreadable request"}"#.into()));
        return;
    }
    let message: Value = match serde_json::from_str(&body) {
        Ok(m) => m,
        Err(e) => {
            let _ = request.respond(reply(400, json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": e.to_string() } }).to_string()));
            return;
        }
    };
    // Notifications and replies need no answer.
    if message.get("method").is_none() || message.get("id").is_none() {
        let _ = request.respond(Response::empty(202));
        return;
    }
    let rpc_id = message["id"].clone();
    let id = next.fetch_add(1, Ordering::Relaxed);
    let (sender, receiver) = sync_channel(1);
    if let Ok(mut w) = waiting.lock() {
        w.insert(id, sender);
    }
    let answered = jobs.send(Job { id, message }).ok().and_then(|()| receiver.recv_timeout(ANSWER_WITHIN).ok());
    if let Ok(mut w) = waiting.lock() {
        w.remove(&id);
    }
    let answered = answered.unwrap_or_else(|| json!({ "jsonrpc": "2.0", "id": rpc_id, "error": { "code": -32000, "message": "Investor did not answer in time. Is the app still open?" } }));
    let _ = request.respond(reply(200, answered.to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post(port: u16, path: &str, headers: &[(&str, &str)], body: &str) -> (u16, String) {
        let agent: ureq::Agent = ureq::Agent::config_builder().http_status_as_error(false).build().into();
        let mut call = agent.post(&format!("http://127.0.0.1:{port}{path}")).header("Content-Type", "application/json");
        for (k, v) in headers {
            call = call.header(*k, *v);
        }
        let mut response = call.send(body).expect("the server answers");
        (response.status().as_u16(), response.body_mut().read_to_string().unwrap_or_default())
    }

    #[test]
    fn hands_requests_to_the_app_and_refuses_strangers() {
        let bridge = Arc::new(Bridge::default());
        let port = 47_999;
        let secret = "0123456789abcdef0123";
        listen(&bridge, port, secret.into()).expect("it starts");
        // The web view's part: answer every request by echoing its method.
        let app = bridge.clone();
        std::thread::spawn(move || {
            while let Ok(job) = app.inbox.lock().unwrap().recv() {
                let reply = json!({ "jsonrpc": "2.0", "id": job.message["id"], "result": { "method": job.message["method"] } });
                if let Some(s) = app.waiting.lock().unwrap().remove(&job.id) {
                    s.send(reply).unwrap();
                }
            }
        });
        let ask = r#"{"jsonrpc":"2.0","id":7,"method":"tools/list"}"#;
        let (status, body) = post(port, &format!("/mcp/{secret}"), &[], ask);
        assert_eq!(status, 200);
        assert_eq!(serde_json::from_str::<Value>(&body).unwrap(), json!({ "jsonrpc": "2.0", "id": 7, "result": { "method": "tools/list" } }));
        let (status, _) = post(port, "/mcp", &[("Authorization", &format!("Bearer {secret}"))], ask);
        assert_eq!(status, 200);
        assert_eq!(post(port, "/mcp/wrong", &[], ask).0, 401);
        assert_eq!(post(port, &format!("/mcp/{secret}"), &[("Origin", "https://example.com")], ask).0, 403);
        assert_eq!(post(port, &format!("/mcp/{secret}"), &[("Host", "evil.example:47999")], ask).0, 403);
        assert_eq!(post(port, &format!("/mcp/{secret}"), &[], r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).0, 202);
        bridge.stop();
    }
}
