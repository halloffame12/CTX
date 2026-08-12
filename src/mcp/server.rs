//! MCP server: stdio + newline-delimited JSON-RPC.

use std::io::{BufRead, Write};

use serde_json::{Value, json};

use crate::errors::CtxResult;
use crate::mcp::protocol::{RpcError, error_response, response};
use crate::mcp::tools::{McpEnv, call_tool, list_tools};

const PROTOCOL_VERSION: &str = "2025-06-18";

pub fn run(env: &McpEnv) -> CtxResult<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    eprintln!("ctx-mcp: server started (protocol v{PROTOCOL_VERSION})");

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let parsed: Result<Value, _> = serde_json::from_str(&line);
        match parsed {
            Ok(raw) => {
                let id = raw.get("id").cloned().unwrap_or(Value::Null);
                let method = raw
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let params = raw.get("params").cloned().unwrap_or_else(|| json!({}));

                // notifications have no id → no response
                if id.is_null() || method.is_empty() {
                    continue;
                }

                let out_msg = match method.as_str() {
                    "initialize" => response(
                        id.clone(),
                        json!({
                            "protocolVersion": PROTOCOL_VERSION,
                            "capabilities": {"tools": {"listChanged": false}},
                            "serverInfo": {"name": "ctx", "version": env!("CARGO_PKG_VERSION")}
                        }),
                    ),
                    "ping" => response(id.clone(), json!({})),
                    "tools/list" => {
                        let tools: Vec<Value> = list_tools()
                            .iter()
                            .map(|t| serde_json::to_value(t).unwrap_or_default())
                            .collect();
                        response(id.clone(), json!({ "tools": tools }))
                    }
                    "tools/call" => match dispatch_call(env, &params) {
                        Ok(result) => response(id.clone(), result),
                        Err(e) => error_response(id.clone(), RpcError::internal(e.to_string())),
                    },
                    "prompts/list" => response(id.clone(), json!({ "prompts": [] })),
                    "resources/list" => response(id.clone(), json!({ "resources": [] })),
                    _ => error_response(
                        id.clone(),
                        RpcError::method_not_found(format!("method not found: {method}")),
                    ),
                };
                let bytes = serde_json::to_vec(&out_msg)?;
                out.write_all(&bytes)?;
                out.write_all(b"\n")?;
                let _ = out.flush();
            }
            Err(e) => {
                let ner = RpcError {
                    code: -32700,
                    message: format!("parse error: {e}"),
                    data: None,
                };
                let bytes = serde_json::to_vec(&error_response(Value::Null, ner))?;
                out.write_all(&bytes)?;
                out.write_all(b"\n")?;
                let _ = out.flush();
            }
        }
    }
    Ok(())
}

fn dispatch_call(env: &McpEnv, params: &Value) -> CtxResult<Value> {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    if name.is_empty() {
        return Ok(json!({
            "content": [ { "type": "text", "text": "missing tool name" } ],
            "isError": true
        }));
    }
    match call_tool(env, name, args) {
        Ok((is_error, text)) => Ok(json!({
            "content": [ { "type": "text", "text": text } ],
            "isError": is_error
        })),
        Err(e) => Ok(json!({
            "content": [ { "type": "text", "text": format!("error running `{name}`: {e}") } ],
            "isError": true
        })),
    }
}
