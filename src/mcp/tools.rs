//! MCP tool definitions and implementations.

use serde_json::{Value, json};

use crate::commands::Project;
use crate::errors::CtxResult;
use crate::mcp::protocol::{TOOL_SCHEMA_VERSION, ToolDef};

pub struct McpEnv {
    pub project: Project,
}

pub fn list_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "ctx_project".into(),
            description: "Return the project overview as JSON: {root, git, files, symbols, dependencies, languages}. Use this first to orient a coding agent on a repo: absolute root path, git root, and how large the codebase is (counts of indexed files, symbols and dependency edges, plus the distinct languages present). Requires the project to have been indexed (see ctx_search for symbol lookup). Returns only counts, never file contents. Prefer ctx_stats for detailed index-health numbers (e.g. index.db size) and ctx_search to actually find symbols.".into(),
            input_schema: json!({"type":"object","properties":{},"additionalProperties":false}),
            schema_version: Some(TOOL_SCHEMA_VERSION),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "root": {"type": "string"},
                    "git": {"type": ["string", "null"]},
                    "files": {"type": "integer"},
                    "symbols": {"type": "integer"},
                    "dependencies": {"type": "integer"},
                    "languages": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["root", "files", "symbols", "dependencies", "languages"]
            })),
        },
        ToolDef {
            name: "ctx_search".into(),
            description: "Search the project's code graph for symbols or file paths by name and return JSON. Symbol results give {name, parent, kind, path, line, signature}; with files=true results give {path, language, size}. Use to find where a function/class/type is defined before reading it, or to locate files by path fragment. Use ctx_symbol for deep detail on a single exact symbol, and ctx_impact to see what depends on a match. query is a case-insensitive substring/name match. Constrain with kind (function, method, class, struct, trait, enum, interface, type, constant, variable, module, field, constructor, impl) and limit results (default 50, 1-500).".into(),
            input_schema: json!({
                "type":"object",
                "properties":{
                    "query":{"type":"string","description":"Case-insensitive substring to match against symbol names or (when files=true) file paths. Required. Empty or very short queries return many results."},
                    "kind":{"type":"string","enum":["function","method","class","interface","type","enum","constant","variable","struct","trait","module","field","constructor","impl"],"description":"Optional: narrow results to this symbol kind. Omit to match any kind."},
                    "files":{"type":"boolean","description":"When true, search file paths instead of symbols and return {path, language, size}. Default false."},
                    "limit":{"type":"integer","minimum":1,"maximum":500,"description":"Maximum number of results. Default 50."}
                },
                "required":["query"]
            }),
            schema_version: Some(TOOL_SCHEMA_VERSION),
            output_schema: Some(json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "parent": {"type": ["string", "null"]},
                        "kind": {"type": "string"},
                        "path": {"type": "string"},
                        "line": {"type": "integer"},
                        "signature": {"type": ["string", "null"]}
                    },
                    "required": ["name", "kind", "path", "line"]
                }
            })),
        },
        ToolDef {
            name: "ctx_skeleton".into(),
            description: "Return a body-less structural skeleton of one source file as {path, language, skeleton}. The skeleton preserves signatures, types, exports and doc comments but strips function bodies, so it is a compact map of a file's public API and structure. Use before editing a file to understand its shape without reading the whole body. path is project-relative or absolute (must be inside the project; traversal is rejected). Set with_stats=true to also include {stats} (symbol counts). Only paths for supported languages can be resolved (error otherwise); use ctx_search (files=true) to confirm a path first.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{
                    "path":{"type":"string","description":"Project-relative or absolute file path. Paths outside the project root are rejected. Required."},
                    "with_stats":{"type":"boolean","description":"When true, include a {stats} field with symbol counts. Default false."}
                },
                "required":["path"]
            }),
            schema_version: Some(TOOL_SCHEMA_VERSION),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "language": {"type": "string"},
                    "skeleton": {"type": "string"},
                    "stats": {"type": ["object", "null"]}
                },
                "required": ["path", "language", "skeleton"]
            })),
        },
        ToolDef {
            name: "ctx_symbol".into(),
            description: "Return deep detail for a single symbol as JSON: [{name, kind, signature, file, line, methods, references, dependencies}]. Use when you already know the exact symbol name and need its definition, signature, methods it exposes, everywhere it is referenced, and its dependencies. For fuzzy or name-based discovery use ctx_search first, then ctx_symbol for the best match. Returns an array because a name may resolve in multiple files.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{"name":{"type":"string","description":"Exact symbol name to look up (e.g. a function, struct, trait or type name). Required."}},
                "required":["name"]
            }),
            schema_version: Some(TOOL_SCHEMA_VERSION),
            output_schema: Some(json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "kind": {"type": "string"},
                        "signature": {"type": ["string", "null"]},
                        "file": {"type": "string"},
                        "line": {"type": "integer"},
                        "methods": {"type": "array", "items": {"type": "object", "properties": {"name": {"type": "string"}, "kind": {"type": "string"}}, "required": ["name", "kind"]}},
                        "references": {"type": "array", "items": {"type": "object", "properties": {"path": {"type": "string"}, "line": {"type": "integer"}}, "required": ["path", "line"]}},
                        "dependencies": {"type": "array", "items": {"type": "object", "properties": {"target": {"type": "string"}, "imported_symbol": {"type": ["string", "null"]}}, "required": ["target"]}}
                    },
                    "required": ["name", "kind", "signature", "file", "line"]
                }
            })),
        },
        ToolDef {
            name: "ctx_dependencies".into(),
            description: "Return the outbound import edges of a file as JSON: [{target, imported_symbol}] — the project files/modules it imports and, where known, the symbol imported. Use to see what a file depends on before refactoring it. path is project-relative or absolute (traversal rejected). For the reverse direction (who imports this) use ctx_dependents. Requires the project to be indexed.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{"path":{"type":"string","description":"Project-relative or absolute file path. Paths outside the project root are rejected. Required."}},
                "required":["path"]
            }),
            schema_version: Some(TOOL_SCHEMA_VERSION),
            output_schema: Some(json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "target": {"type": "string"},
                        "imported_symbol": {"type": ["string", "null"]}
                    },
                    "required": ["target"]
                }
            })),
        },
        ToolDef {
            name: "ctx_dependents".into(),
            description: "Return the inbound (reverse) dependency edges of a file as JSON: [{source, imported_symbol}] — the project files that import it and, where known, the symbol they import. Use to find every consumer of a file before changing or removing it. path is project-relative or absolute (traversal rejected). For the forward direction (what a file imports) use ctx_dependencies.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{"path":{"type":"string","description":"Project-relative or absolute file path. Paths outside the project root are rejected. Required."}},
                "required":["path"]
            }),
            schema_version: Some(TOOL_SCHEMA_VERSION),
            output_schema: Some(json!({
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "source": {"type": "string"},
                        "imported_symbol": {"type": ["string", "null"]}
                    },
                    "required": ["source"]
                }
            })),
        },
        ToolDef {
            name: "ctx_impact".into(),
            description: "Analyze the blast radius of changing a symbol or file. Returns an ImpactReport: {target, target_symbol, depth, direct, indirect, tests, unknown} where direct/indirect/test lists are [{path, distance, symbols}]; unknown surfaces imports that could not be statically mapped. Use before modifying code to estimate what else must be checked or updated. Provide exactly one of symbol (a name) or path (a project-relative file); depth controls how many hops of indirect impact to traverse (default 3, 1-20). Requires the project to be indexed; returns an error object if the target is not found.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{
                    "symbol":{"type":"string","description":"Symbol name to analyze. Provide this OR path, not both."},
                    "path":{"type":"string","description":"Project-relative file path to analyze, as an alternative to symbol."},
                    "depth":{"type":"integer","minimum":1,"maximum":20,"description":"How many hops of indirect impact to traverse. Default 3."}
                }
            }),
            schema_version: Some(TOOL_SCHEMA_VERSION),
            output_schema: Some(json!({"type": "object"})),
        },
        ToolDef {
            name: "ctx_context".into(),
            description: "Build a compact, relevance-ranked context package for a coding task and return it as JSON. This is the high-value tool: give it a natural-language task and it returns the most relevant files/symbols/snippets to feed to an LLM, optionally including recent git changes. Use when you need a focused slice of the codebase for a prompt instead of reading many files. task is required and should describe the goal; include_bodies (default false) embeds function bodies; max_tokens (default auto, 128-100000) caps package size.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{
                    "task":{"type":"string","description":"Natural-language description of the coding task the context should support. Required."},
                    "include_bodies":{"type":"boolean","description":"When true, include function/body text in the package (larger output). Default false."},
                    "max_tokens":{"type":"integer","minimum":128,"maximum":100000,"description":"Upper bound on package size in tokens. Omit for automatic sizing."}
                },
                "required":["task"]
            }),
            schema_version: Some(TOOL_SCHEMA_VERSION),
            output_schema: Some(json!({"type": "object"})),
        },
        ToolDef {
            name: "ctx_changed".into(),
            description: "Return files and symbols changed since a git reference as JSON, including working-tree changes by default. Use to focus an agent on what changed in a branch or commit range before reviewing or testing. ref is a git ref (e.g. HEAD~5, main, a SHA); omit it to report uncommitted working-tree changes. Only works in git repositories (returns an error otherwise). For a per-symbol semantic diff between two refs use ctx_diff; for a plain list of changed file paths use this tool.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{"ref":{"type":"string","description":"Git ref to diff against, e.g. HEAD, main, HEAD~5, or a commit SHA. Omit to report working-tree (uncommitted) changes."}}
            }),
            schema_version: Some(TOOL_SCHEMA_VERSION),
            output_schema: Some(json!({"type": "object"})),
        },
        ToolDef {
            name: "ctx_diff".into(),
            description: "Return a semantic symbol diff between two git refs as JSON: the symbols added, modified or removed per file. Compared to ctx_changed, this is a structural (symbol-aware) diff rather than a file list. Provide base and head explicitly, or supply only base (or neither) — when base alone is given it resolves to its merge-base with HEAD so the diff shows only the current branch's changes; head defaults to HEAD. Only works in git repositories.".into(),
            input_schema: json!({
                "type":"object",
                "properties":{
                    "base":{"type":"string","description":"Base git ref. Optional: if only base (or neither) is given, base resolves to its merge-base with HEAD, showing only the current branch's additions/modifications/removals. Defaults to HEAD."},
                    "head":{"type":"string","description":"Head git ref to compare against base. Defaults to HEAD."}
                }
            }),
            schema_version: Some(TOOL_SCHEMA_VERSION),
            output_schema: Some(json!({"type": "object"})),
        },
        ToolDef {
            name: "ctx_stats".into(),
            description: "Return index-health statistics as JSON: counts of indexed files, symbols and dependency edges, plus the size of index.db (the underlying code-graph database). Use to check whether the project has been indexed (all-zero counts mean you must run init/indexing before graph tools such as ctx_search or ctx_impact will return results). Read-only, no side effects. For a broader project overview (root/git/languages) use ctx_project.".into(),
            input_schema: json!({"type":"object","properties":{},"additionalProperties":false}),
            schema_version: Some(TOOL_SCHEMA_VERSION),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "root": {"type": "string"},
                    "files": {"type": "integer"},
                    "symbols": {"type": "integer"},
                    "dependencies": {"type": "integer"},
                    "db_size": {"type": "integer"}
                },
                "required": ["root", "files", "symbols", "dependencies", "db_size"]
            })),
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
            .and_then(|git| crate::git::changed::changed_files(git, None, true).ok())
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
