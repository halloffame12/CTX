//! JSON-RPC 2.0 message types for MCP over stdio.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMessage {
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    pub fn invalid_params(msg: impl Into<String>) -> RpcError {
        RpcError {
            code: -32602,
            message: msg.into(),
            data: None,
        }
    }
    pub fn method_not_found(msg: impl Into<String>) -> RpcError {
        RpcError {
            code: -32601,
            message: msg.into(),
            data: None,
        }
    }
    pub fn internal(msg: impl Into<String>) -> RpcError {
        RpcError {
            code: -32603,
            message: msg.into(),
            data: None,
        }
    }
}

/// Schema version for MCP tool definitions. Increment when making breaking changes to tool schemas.
pub const TOOL_SCHEMA_VERSION: u32 = 1;

/// JSON Schema for a tool input parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParamSchema {
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

/// The MCP `tools/list` result schema.
#[derive(Debug, Clone, Serialize)]
pub struct ToolsListResult {
    pub tools: Vec<ToolDef>,
}

/// MCP tool definition with versioned schema.
#[derive(Debug, Clone, Serialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    /// Schema version for this tool. Defaults to TOOL_SCHEMA_VERSION.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    /// Optional output schema for the tool result.
    #[serde(skip_serializing_if = "Option::is_none", rename = "outputSchema")]
    pub output_schema: Option<Value>,
}

/// MCP tool call result with structured output schema.
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallResult {
    pub content: Vec<ContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "isError")]
    pub is_error: Option<bool>,
    /// Structured output schema for the tool result (optional, for schema-aware clients).
    #[serde(skip_serializing_if = "Option::is_none", rename = "outputSchema")]
    pub output_schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

pub fn response(id: Value, result: Value) -> RpcMessage {
    RpcMessage {
        jsonrpc: Some("2.0".to_string()),
        id: Some(id),
        method: None,
        params: None,
        result: Some(result),
        error: None,
    }
}

pub fn error_response(id: Value, error: RpcError) -> RpcMessage {
    RpcMessage {
        jsonrpc: Some("2.0".to_string()),
        id: Some(id),
        method: None,
        params: None,
        result: None,
        error: Some(error),
    }
}

pub fn notification(method: &str, params: Value) -> RpcMessage {
    RpcMessage {
        jsonrpc: Some("2.0".to_string()),
        id: None,
        method: Some(method.to_string()),
        params: Some(params),
        result: None,
        error: None,
    }
}
