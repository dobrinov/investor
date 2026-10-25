//! An MCP server over stdio that lets an AI assistant read one `.invest`
//! file, run the app's calculations, and record changes the person agreed
//! to. Every change keeps the previous version for undo; `--read-only`
//! turns writing off.
//!
//! The password never passes through the conversation. It comes from, in
//! order: `INVESTOR_PASSWORD_COMMAND` (a shell command that prints it), the
//! macOS Keychain (see `investor-mcp remember-password`), or
//! `INVESTOR_PASSWORD`.

mod book;
mod tools;

use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use book::Book;

const PROTOCOL_VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];


fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut file = std::env::var_os("INVESTOR_FILE").map(PathBuf::from);
    let mut remember = false;
    let mut writes = std::env::var_os("INVESTOR_READ_ONLY").is_none_or(|v| v.is_empty() || v == "0");
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--file" => file = Some(args.next().context("--file needs a path")?.into()),
            "remember-password" => remember = true,
            "--read-only" => writes = false,
            "--help" | "-h" => {
                println!("{}", include_str!("usage.txt"));
                return Ok(());
            }
            other => bail!("unknown argument {other:?}; see --help"),
        }
    }
    let file = file.context("say which file to read: --file /path/to/portfolio.invest (or set INVESTOR_FILE)")?;
    if remember {
        return book::remember_password(&file);
    }
    serve(Book::new(file), writes)
}

/// Newline-delimited JSON-RPC on stdin and stdout.
fn serve(mut book: Book, writes: bool) -> Result<()> {
    let stdout = io::stdout();
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(message) => handle(&mut book, &message, writes),
            Err(e) => Some(json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": e.to_string() } })),
        };
        if let Some(reply) = reply {
            let mut out = stdout.lock();
            serde_json::to_writer(&mut out, &reply)?;
            out.write_all(b"\n")?;
            out.flush()?;
        }
    }
    Ok(())
}

/// Answers a request; notifications get no reply.
fn handle(book: &mut Book, message: &Value, writes: bool) -> Option<Value> {
    let id = message.get("id")?.clone();
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    let result = match message.get("method").and_then(Value::as_str).unwrap_or_default() {
        "initialize" => {
            let asked = params["protocolVersion"].as_str().unwrap_or_default();
            let version = PROTOCOL_VERSIONS.iter().find(|v| **v == asked).unwrap_or(&PROTOCOL_VERSIONS[0]);
            Ok(json!({
                "protocolVersion": version,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "investor", "version": env!("CARGO_PKG_VERSION") },
                "instructions": if writes { investor_core::assistant::INSTRUCTIONS.to_owned() } else { format!("{} This server is read-only: the person records changes in the Investor app.", investor_core::assistant::INSTRUCTIONS) },
            }))
        }
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tools::list(writes) })),
        "tools/call" => {
            let name = params["name"].as_str().unwrap_or_default();
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            // Tool failures go back to the model as text it can act on.
            Ok(match tools::call(book, name, &args, writes) {
                Ok(value) => json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&value).unwrap_or_default() }] }),
                Err(e) => json!({ "content": [{ "type": "text", "text": format!("{e:#}") }], "isError": true }),
            })
        }
        method => Err(json!({ "code": -32601, "message": format!("unknown method {method}") })),
    };
    Some(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(error) => json!({ "jsonrpc": "2.0", "id": id, "error": error }),
    })
}
