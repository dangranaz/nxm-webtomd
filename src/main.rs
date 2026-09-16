//! nxm-webtomd — a tiny MCP server that fetches web articles and converts them to Markdown.
//!
//! Transport: JSON-RPC over stdio (one JSON object per line), the transport used
//! by CLI agents such as Kiro CLI, Claude Code and Cursor.

mod convert;
mod handler;
mod protocol;

use std::io::{self, BufRead, Write};

use crate::protocol::{codes, JsonRpcRequest, JsonRpcResponse};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("stdin read error: {e}");
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                write_response(
                    &stdout,
                    &JsonRpcResponse::error(None, codes::PARSE_ERROR, format!("parse error: {e}")),
                );
                continue;
            }
        };

        // Notifications return None and must not be answered.
        if let Some(response) = handler::dispatch(&request) {
            write_response(&stdout, &response);
        }
    }
}

fn write_response(stdout: &io::Stdout, response: &JsonRpcResponse) {
    match serde_json::to_string(response) {
        Ok(s) => {
            let mut out = stdout.lock();
            let _ = writeln!(out, "{s}");
            let _ = out.flush();
        }
        Err(e) => eprintln!("failed to serialize response: {e}"),
    }
}