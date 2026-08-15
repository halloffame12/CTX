//! MCP tool definitions and implementations.

use serde_json::{Value, json};

use crate::commands::Project;
use crate::errors::CtxResult;
use crate::mcp::protocol::ToolDef;

pub struct McpEnv {
    pub project: Project,
}

pub fn list_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "ctx_project".into(),
            description: "Return the project overview: root, git, and counts of indexed files, symbols and dependencies.".into(),
            input_schema: json!({"type":"object","properties":{},"additionalProperties":false}),
        },
        ToolDef {
            name: "ctx_search".into(),
            description: "Search the code graph for symbols or files by name.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{
                    "query":{"type":"string"},
                    "kind":{"type":"string","enum":["function","method","class","interface","type","enum","constant","variable","struct","trait","module","field","constructor","impl"]},
                    "files":{"type":"boolean","description":"search file paths instead of symbols"},
                    "limit":{"type":"integer","minimum":1,"maximum":500}
                },
                "required":["query"]
            }),
        },
        ToolDef {
            name: "ctx_skeleton".into(),
            description: "Return a body-less structural skeleton of a source file, preserving signatures, types and exports.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{
                    "path":{"type":"string","description":"project-relative or absolute file path"},
                    "with_stats":{"type":"boolean"}
                },
                "required":["path"]
            }),
        },
        ToolDef {
            name: "ctx_symbol".into(),
            description: "Details about a symbol: definition, kind, methods, references and dependencies.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{"name":{"type":"string"}},
                "required":["name"]
            }),
        },
        ToolDef {
            name: "ctx_dependencies".into(),
            description: "List the files and modules a given file imports.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{"path":{"type":"string"}},
                "required":["path"]
            }),
        },
        ToolDef {
            name: "ctx_dependents".into(),
            description: "List files that import a given file (reverse dependencies).".into(),
            input_schema: json!({
                "type":"object",
                "properties":{"path":{"type":"string"}},
                "required":["path"]
            }),
        },
        ToolDef {
            name: "ctx_impact".into(),
            description: "Analyze files and symbols potentially affected by changing a symbol or file.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{
                    "symbol":{"type":"string","description":"symbol name"},
                    "path":{"type":"string","description":"project-relative file path (alternative to symbol)"},
                    "depth":{"type":"integer","minimum":1,"maximum":20}
                }
            }),
        },
        ToolDef {
            name: "ctx_context".into(),
            description: "Build a compact, relevance-ranked context package for a coding task.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{
                    "task":{"type":"string"},
                    "include_bodies":{"type":"boolean"},
                    "max_tokens":{"type":"integer","minimum":128,"maximum":100000}
                },
                "required":["task"]
            }),
        },
        ToolDef {
            name: "ctx_changed".into(),
            description: "List files and symbols changed since a git reference (default: working tree).".into(),
            input_schema: json!({
                "type":"object",
                "properties":{"ref":{"type":"string","description":"git ref, e.g. HEAD, main, HEAD~5"}}
            }),
        },
        ToolDef {
            name: "ctx_diff".into(),
            description: "Semantic diff: symbols added, modified or removed between two refs. Default base is HEAD.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{
                    "base":{"type":"string"},
                    "head":{"type":"string"}
                }
            }),
        },
        ToolDef {
            name: "ctx_stats".into(),
            description: "Index statistics: counts of files, symbols and dependencies, and the size of index.db.".into(),
            input_schema: json!({"type":"object","properties":{},"additionalProperties":false}),
        },
    ]
}

/// Execute a tool call. Returns (is_error, output_text).
pub fn call_tool(env: &McpEnv, name: &str, args: Value) -> CtxResult<(bool, String)> {
    let project = &env.project;
    let text = match name {
        "ctx_project" => serde_json::to_string_pretty(&crate::commands::project_summary(project)?)?,
        "ctx_search" => tool_search(project, &args)?,
        "ctx_skeleton" => tool_skeleton(project, &args)?,
        "ctx_symbol" => tool_symbol(project, &args)?,
        "ctx_dependencies" => tool_deps_out(project, &args)?,
        "ctx_dependents" => tool_deps_in(project, &args)?,
        "ctx_impact" => tool_impact(project, &args)?,
        "ctx_context" => tool_context(project, &args)?,
        "ctx_changed" => tool_changed(project, &args)?,
        "ctx_diff" => tool_diff(project, &args)?,
        "ctx_stats" => crate::commands::stats::stats_json(&project.root, &project.db)?,
        _ => {
            return Err(crate::errors::CtxError::Other(format!(
                "unknown tool `{name}`"
            )));
        }
    };
    Ok((false, text))
}

fn str_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn bool_arg(args: &Value, key: &str) -> bool {
    args.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn int_arg(args: &Value, key: &str, default: i64) -> i64 {
    args.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}

fn tool_search(project: &Project, args: &Value) -> CtxResult<String> {
    let query = str_arg(args, "query")
        .ok_or_else(|| crate::errors::CtxError::Other("missing `query`".into()))?;
    let kind = str_arg(args, "kind");
    let files = bool_arg(args, "files");
    let limit = int_arg(args, "limit", 50) as usize;
    if files {
        let files = project.db.files_like(&query, limit)?;
        let v: Vec<Value> = files
            .iter()
            .map(|f| json!({"path": f.path, "language": f.language, "size": f.size}))
            .collect();
        return Ok(serde_json::to_string_pretty(&v)?);
    }
    let symbols = project.db.search(&query, kind.as_deref(), limit)?;
    let v: Vec<Value> = symbols
        .iter()
        .map(|s| {
            let path = project
                .db
                .file_by_id(s.file_id)
                .ok()
                .flatten()
                .map(|f| f.path)
                .unwrap_or_default();
            json!({
                "name": s.name,
                "parent": s.parent,
                "kind": s.kind,
                "path": path,
                "line": s.start_line,
                "signature": s.signature,
            })
        })
        .collect();
    Ok(serde_json::to_string_pretty(&v)?)
}

fn tool_skeleton(project: &Project, args: &Value) -> CtxResult<String> {
    let path = str_arg(args, "path")
        .ok_or_else(|| crate::errors::CtxError::Other("missing `path`".into()))?;
    let rel = crate::commands::normalize_rel_path(&project.root, &path)?;
    let language = crate::lang::language_of_path(std::path::Path::new(&rel))
        .ok_or_else(|| crate::errors::CtxError::UnsupportedLanguage(rel.clone()))?;
    let source = std::fs::read_to_string(project.root.join(&rel))?;
    let result = crate::context::skeleton::skeleton_for(&project.root, &rel, language, &source)?;
    let mut v = json!({"path": rel, "language": language.as_str(), "skeleton": result.skeleton});
    if bool_arg(args, "with_stats") {
        v["stats"] = serde_json::to_value(&result.stats)?;
    }
    Ok(serde_json::to_string_pretty(&v)?)
}

fn tool_symbol(project: &Project, args: &Value) -> CtxResult<String> {
    let name = str_arg(args, "name")
        .ok_or_else(|| crate::errors::CtxError::Other("missing `name`".into()))?;
    let details = crate::graph::symbols::symbol_detail(&project.db, &name)?;
    let v: Vec<Value> = details
        .iter()
        .map(|d| {
            json!({
                "name": d.symbol.name,
                "kind": d.symbol.kind,
                "signature": d.symbol.signature,
                "file": d.file.path,
                "line": d.symbol.start_line,
                "methods": d.methods,
                "references": d.references,
                "dependencies": d.dependencies,
            })
        })
        .collect();
    Ok(serde_json::to_string_pretty(&v)?)
}

fn tool_deps_out(project: &Project, args: &Value) -> CtxResult<String> {
    let path = str_arg(args, "path")
        .ok_or_else(|| crate::errors::CtxError::Other("missing `path`".into()))?;
    let rel = crate::commands::normalize_rel_path(&project.root, &path)?;
    let file = crate::commands::resolve_file(project, &rel)?;
    let deps = project.db.internal_dependencies_of(file.id)?;
    let v: Vec<Value> = deps
        .iter()
        .map(|(p, sym)| json!({"target": p, "imported_symbol": sym}))
        .collect();
    Ok(serde_json::to_string_pretty(&v)?)
}

fn tool_deps_in(project: &Project, args: &Value) -> CtxResult<String> {
    let path = str_arg(args, "path")
        .ok_or_else(|| crate::errors::CtxError::Other("missing `path`".into()))?;
    let rel = crate::commands::normalize_rel_path(&project.root, &path)?;
    let file = crate::commands::resolve_file(project, &rel)?;
    let v: Vec<Value> = project
        .db
        .dependents_of(file.id)?
        .iter()
        .map(|(p, sym)| json!({"source": p, "imported_symbol": sym}))
        .collect();
    Ok(serde_json::to_string_pretty(&v)?)
}

fn tool_impact(project: &Project, args: &Value) -> CtxResult<String> {
    let symbol = str_arg(args, "symbol");
    let path = str_arg(args, "path");
    let depth = int_arg(args, "depth", 3).clamp(1, 20) as u32;
    let target = match (&symbol, &path) {
        (Some(s), _) => s.clone(),
        (None, Some(p)) => crate::commands::normalize_rel_path(&project.root, p)?,
        _ => {
            return Err(crate::errors::CtxError::Other(
                "provide either `symbol` or `path`".into(),
            ));
        }
    };
    if let Some((found_path, id, symbol)) =
        crate::graph::impact::resolve_target(&project.db, &target)?
    {
        let report =
            crate::graph::impact::impact(&project.db, &found_path, id, symbol.as_deref(), depth)?;
        return Ok(serde_json::to_string_pretty(&report)?);
    }
    Ok(serde_json::to_string_pretty(&json!({
        "error": format!("target `{target}` not found in the graph"),
        "hint": "run `ctx init` to index the project, or try a symbol name"
    }))?)
}

fn tool_context(project: &Project, args: &Value) -> CtxResult<String> {
    let task = str_arg(args, "task")
        .ok_or_else(|| crate::errors::CtxError::Other("missing `task`".into()))?;
    let include_bodies = bool_arg(args, "include_bodies");
    let max_tokens = args
        .get("max_tokens")
        .and_then(|v| v.as_i64())
        .map(|n| n as usize);
    let git_changes: Option<Vec<String>> = if project.git.is_some() {
        project
            .git
            .as_ref()
            .and_then(|git| crate::git::changed::changed_files(git, None).ok())
            .map(|files| files.into_iter().map(|c| c.path).collect())
    } else {
        None
    };
    let package = crate::context::build_context_with(
        &project.db,
        &project.root,
        &task,
        &project.config,
        include_bodies,
        max_tokens,
        git_changes.as_deref(),
    )?;
    Ok(serde_json::to_string_pretty(&package)?)
}

fn tool_changed(project: &Project, args: &Value) -> CtxResult<String> {
    let Some(git) = &project.git else {
        return Ok(serde_json::to_string_pretty(
            &json!({"error": "not a git repository"}),
        )?);
    };
    let since = str_arg(args, "r#ref").or_else(|| str_arg(args, "ref"));
    let report = crate::git::changed::changed_symbols(git, &project.db, since.as_deref())?;
    Ok(serde_json::to_string_pretty(&report)?)
}

fn tool_diff(project: &Project, args: &Value) -> CtxResult<String> {
    let Some(git) = &project.git else {
        return Ok(serde_json::to_string_pretty(
            &json!({"error": "not a git repository"}),
        )?);
    };
    let base = str_arg(args, "base");
    let head = str_arg(args, "head");
    let diff =
        crate::git::diff::symbol_diff(git, base.as_deref(), head.as_deref(), Some(&project.root))?;
    Ok(serde_json::to_string_pretty(&diff)?)
}
