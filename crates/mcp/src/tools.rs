//! The tools an MCP client can call: the core's assistant tools, plus undo.
//! Reads use the file as it is on disk; each write re-reads it, keeps the
//! previous version in the history folder, and saves.

use std::path::PathBuf;

use anyhow::Result;
use chrono::Local;
use investor_core::assistant::{self, Env};
use investor_core::jurisdiction::JurisdictionPack;
use serde_json::{Value, json};

use crate::book::Book;

/// The tool list in MCP's shape. Read-only servers list no writes.
pub fn list(writes: bool) -> Value {
    let mut tools: Vec<Value> = assistant::tools(writes)
        .as_array()
        .into_iter()
        .flatten()
        .map(|t| json!({ "name": t["name"], "description": t["description"], "inputSchema": t["input_schema"] }))
        .collect();
    if writes {
        tools.push(json!({
            "name": "undo_last_change",
            "description": "Puts the file back as it was before the last change made through these tools. Each undo goes one change further back.",
            "inputSchema": { "type": "object", "properties": {} }
        }));
    }
    Value::Array(tools)
}

pub fn call(book: &mut Book, name: &str, args: &Value, writes: bool) -> Result<Value> {
    if name == "undo_last_change" && writes {
        return book.undo();
    }
    let stamp = Local::now().format("%Y%m%d%H%M%S%f").to_string();
    let lookup = |country: &str, year: i32| pack(country, year);
    let env = Env { today: Local::now().date_naive(), stamp, pack: &lookup };
    if assistant::is_write(name) {
        if !writes {
            anyhow::bail!("this server is read-only (started with --read-only); record the change in the Investor app");
        }
        return book.change(|vault| assistant::call(vault, name, args, &env).map_err(Into::into));
    }
    let mut vault = book.vault()?.clone();
    Ok(assistant::call(&mut vault, name, args, &env)?)
}

fn packs_dir() -> PathBuf {
    std::env::var_os("INVESTOR_JURISDICTIONS").map_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../jurisdictions")), PathBuf::from)
}

fn pack(country: &str, year: i32) -> Option<JurisdictionPack> {
    let text = std::fs::read_to_string(packs_dir().join(country.to_lowercase()).join(format!("{year}.toml"))).ok()?;
    JurisdictionPack::from_toml(&text).ok()
}
