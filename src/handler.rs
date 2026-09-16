//! MCP request handling: `initialize`, `tools/list`, `tools/call`.
//!
//! The tool *interface* (names, input schemas) lives here and is the public
//! contract of this server. The actual work is delegated to [`crate::convert`].

use serde_json::{json, Value};

use crate::convert;
use crate::protocol::{codes, JsonRpcRequest, JsonRpcResponse};

/// MCP protocol version this server implements.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// Dispatch a single JSON-RPC request to the right handler.
//
// Returns `None` for notifications (requests without an `id`), which must not
// produce a response per JSON-RPC.
pub fn dispatch(req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
    // Notifications carry no id and expect no reply (e.g. `notifications/initialized`).
    if req.id.is_none() {
        return None;
    }
    let id = req.id.clone();

    let resp = match req.method.as_str() {
        "initialize" => JsonRpcResponse::ok(id, initialize_result()),
        "tools/list" => JsonRpcResponse::ok(id, tools_list_result()),
        "tools/call" => handle_tools_call(id, &req.params),
        other => JsonRpcResponse::error(
            id,
            codes::METHOD_NOT_FOUND,
            format!("method not found: {other}"),
        ),
    };
    Some(resp)
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": { "tools": {} },
        "serverInfo": {
            "name": "nxm-webtomd",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

/// The public tool contract. This is exactly what an agent sees via `tools/list`.
fn tools_list_result() -> Value {
    json!({
        "tools": [
            {
                "name": "web_to_md",
                "description": "Fetch a web article and extract its main content as clean Markdown. \
Applies a strict fetch policy (timeout, size limit, SSRF protection, content-type validation) \
and strips active content to mitigate prompt‑injection risks.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL of the article to fetch."
                        },
                        "output_path": {
                            "type": "string",
                            "description": "Optional path to write the .md result. If omitted, the Markdown is returned inline."
                        }
                    },
                    "required": ["url"]
                }
            }
        ]
    })
}

fn handle_tools_call(id: Option<Value>, params: &Value) -> JsonRpcResponse {
    let name = match params.get("name").and_then(Value::as_str) {
        Some(n) => n,
        None => {
            return JsonRpcResponse::error(id, codes::INVALID_PARAMS, "missing tool 'name'")
        }
    };
    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    let result = match name {
        "web_to_md" => tool_web_to_md(&args),
        other => Err(format!("unknown tool: {other}")),
    };

    match result {
        Ok(text) => JsonRpcResponse::ok(id, tool_text_result(&text)),
        Err(msg) => JsonRpcResponse::ok(id, tool_error_result(&msg)),
    }
}

fn tool_web_to_md(args: &Value) -> Result<String, String> {
    let url = require_str(args, "url")?;
    let md = convert::url_to_md(&url).map_err(|e| e.to_string())?;
    match args.get("output_path").and_then(Value::as_str) {
        Some(out) => {
            std::fs::write(out, &md).map_err(|e| format!("failed to write {out}: {e}"))?;
            Ok(format!("Wrote {} bytes of Markdown to {out}", md.len()))
        }
        None => Ok(md),
    }
}

fn require_str(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .ok_or_else(|| format!("missing required argument '{key}'"))
}

/// MCP `tools/call` success payload (content array of text).
fn tool_text_result(text: &str) -> Value {
    json!({
        "content": [ { "type": "text", "text": text } ],
        "isError": false
    })
}

/// MCP `tools/call` error payload (isError = true, still a valid result).
fn tool_error_result(message: &str) -> Value {
    json!({
        "content": [ { "type": "text", "text": message } ],
        "isError": true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_list_advertises_tool() {
        let v = tools_list_result();
        let names: Vec<&str> = v["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"web_to_md"));
    }

    #[test]
    fn notification_produces_no_response() {
        let req = JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: None,
            method: "notifications/initialized".into(),
            params: json!({}),
        };
        assert!(dispatch(&req).is_none());
    }

    #[test]
    fn unknown_method_is_method_not_found() {
        let req = JsonRpcRequest {
            jsonrpc: Some("2.0".into()),
            id: Some(json!(1)),
            method: "does/not/exist".into(),
            params: json!({}),
        };
        let resp = dispatch(&req).unwrap();
        assert_eq!(resp.error.unwrap().code, codes::METHOD_NOT_FOUND);
    }

    #[test]
    fn web_to_md_missing_arg_is_tool_error() {
        let resp = handle_tools_call(Some(json!(1)), &json!({"name": "web_to_md", "arguments": {}}));
        let result = resp.result.unwrap();
        assert_eq!(result["isError"], json!(true));
    }
}