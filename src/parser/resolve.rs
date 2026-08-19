use std::path::{Component, Path, PathBuf};

use crate::lang::LanguageId;
use crate::parser::traits::ResolvedDependency;

const PROBE_EXTS: &[&str] = &[
    "ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs", "py", "pyi", "rs", "go",
];

pub fn normalize_rel(path: &str) -> String {
    path.replace('\\', "/")
}

/// The directory (project-relative) containing `rel_path`.
pub fn dir_of(rel_path: &str) -> String {
    match rel_path.rfind('/') {
        Some(i) => rel_path[..i].to_string(),
        None => String::new(),
    }
}

/// Resolve a full filesystem path relative to `current_dir` (which is relative
/// to `root`) for an import specifier, probing common extensions and index
/// files. Returns a project-relative path on success.
fn probe(root: &Path, base: &Path, spec: &str) -> Option<String> {
    if spec.is_empty() {
        return None;
    }
    let candidates = file_candidates(spec);
    for cand in &candidates {
        let p = base.join(cand);
        // If spec itself was the exact path, that counts.
        if p.is_file() {
            return relpath_by_join(root, base, cand);
        }
        // Probe extensions on the last segment. Only skip when the spec already
        // carries a *known source extension*: dotted stems like `user.repository`
        // must still be extended (`user.repository` → `user.repository.ts`).
        let has_real_ext = p
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| PROBE_EXTS.contains(&e));
        if !has_real_ext {
            for ext in PROBE_EXTS {
                let s = format!("{cand}.{ext}");
                if base.join(&s).is_file() {
                    return relpath_by_join(root, base, &s);
                }
            }
        }
        // directory index
        for idx in ["index", "mod", "__init__"] {
            for ext in PROBE_EXTS {
                let q = p.join(format!("{idx}.{ext}"));
                if q.is_file() {
                    let s = format!("{cand}/{idx}.{ext}");
                    return relpath_by_join(root, base, &s);
                }
            }
        }
    }
    None
}

/// Probe for a relative import against a project-relative directory
/// (`base_rel_dir`, empty string for project root).
pub fn probe_rel(root: &Path, base_rel_dir: &str, spec: &str) -> Option<String> {
    let base = root.join(base_rel_dir);
    probe(root, &base, spec)
}

fn file_candidates(spec: &str) -> Vec<String> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if trimmed.ends_with('/') {
        return vec![trimmed.trim_end_matches('/').to_string()];
    }
    vec![
        trimmed.to_string(),
        format!("{}/index", trimmed),
        format!("{}/mod", trimmed),
    ]
}

/// relative to current dir -> project-relative via join (no normalization
/// beyond what's present).
fn relpath_by_join(root: &Path, base: &Path, cand: &str) -> Option<String> {
    let joined = base.join(cand);
    match joined.strip_prefix(root) {
        Ok(rel) => Some(collapse_dots(&normalize_rel(&rel.to_string_lossy()))),
        Err(_) => {
            let text = normalize_rel(&joined.to_string_lossy());
            Some(collapse_dots(&text))
        }
    }
}

/// Resolve an import spec to a project-relative path, external module or
/// unresolved reference.
///
/// `current_rel` is the importing file's path relative to `root`.
pub fn resolve_import(root: &Path, current_rel: &str, spec_raw: &str) -> ResolvedDependency {
    let spec = spec_raw.trim();
    if spec.is_empty() {
        return ResolvedDependency::Unresolved(spec_raw.to_string());
    }

    let base = root.join(dir_of(current_rel));

    if spec.starts_with("./") || spec.starts_with("../") {
        if let Some(rel) = probe(root, &base, spec)
            && inside_root(root, &rel)
        {
            return ResolvedDependency::Internal(rel);
        }
        return ResolvedDependency::Unresolved(spec.to_string());
    }

    if spec.starts_with('/') {
        // Root-relative import ("/src/lib/util")
        let root_base = root.to_path_buf();
        if let Some(rel) = probe(root, &root_base, spec.trim_start_matches('/'))
            && inside_root(root, &rel)
        {
            return ResolvedDependency::Internal(rel);
        }
        return ResolvedDependency::Unresolved(spec.to_string());
    }

    // Node-style aliases like "@/lib/util" (root-relative). The `@/` prefix
    // conventionally maps to the project `src/` directory (Vite/Next.js), so
    // probe both the repo root and `src/`.
    if let Some(rest) = spec.strip_prefix('@').and_then(|r| r.strip_prefix('/')) {
        for base in [root.to_path_buf(), root.join("src")] {
            if let Some(rel) = probe(root, &base, rest)
                && inside_root(root, &rel)
            {
                return ResolvedDependency::Internal(rel);
            }
        }
        return ResolvedDependency::Unresolved(spec.to_string());
    }

    // Bare specifier: could be a baseUrl-relative import or an external dep.
    if let Some(rel) = probe(root, root, spec)
        && inside_root(root, &rel)
    {
        return ResolvedDependency::Internal(rel);
    }

    // Workspace package (monorepo): `import "@acme/core"` resolves to that
    // package's entry file when it is listed in the root package.json
    // `workspaces`.
    if let Some(rel) = resolve_workspace_package(root, spec) {
        return ResolvedDependency::Internal(rel);
    }

    ResolvedDependency::External(spec.to_string())
}

/// Resolve a bare package specifier to a workspace member's entry file.
///
/// Reads the root `package.json` (or `pnpm-workspace.yaml` presence is not
/// handled here) for a `workspaces` array, walks each member directory, and
/// matches the member's `package.json` `name`. The entry file is chosen from
/// `main`, `module`, `types`, `exports.import`/`exports.require` (string
/// form), then falls back to `src/index.{ts,tsx,js,jsx}`.
fn resolve_workspace_package(root: &Path, spec: &str) -> Option<String> {
    let pkg_path = root.join("package.json");
    let content = std::fs::read_to_string(&pkg_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let workspaces = match json.get("workspaces") {
        Some(serde_json::Value::Array(ws)) => ws.clone(),
        Some(serde_json::Value::Object(obj)) => {
            obj.get("packages").and_then(|p| p.as_array()).cloned()?
        }
        _ => return None,
    };
    for ws in workspaces {
        let pattern = ws.as_str()?;
        let base_dir = pattern.trim_end_matches('*').trim_end_matches('/');
        if base_dir.is_empty() {
            continue;
        }
        let dir = root.join(base_dir);
        let Ok(read) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in read.flatten() {
            let member = entry.path();
            if !member.is_dir() {
                continue;
            }
            let member_pkg = member.join("package.json");
            let Ok(member_content) = std::fs::read_to_string(&member_pkg) else {
                continue;
            };
            let Ok(member_json) = serde_json::from_str::<serde_json::Value>(&member_content) else {
                continue;
            };
            let name = member_json
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or_default();
            if name != spec {
                continue;
            }
            return workspace_entry(root, &member);
        }
    }
    None
}

/// Pick the entry file for a workspace member package, probing the declared
/// entry points then conventional `src/index.*`.
fn workspace_entry(root: &Path, member: &Path) -> Option<String> {
    let member_pkg = member.join("package.json");
    let content = std::fs::read_to_string(&member_pkg).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let mut candidates: Vec<String> = Vec::new();
    for key in ["main", "module", "types"] {
        if let Some(v) = json.get(key).and_then(|v| v.as_str()) {
            candidates.push(v.to_string());
        }
    }
    if let Some(exports) = json.get("exports") {
        if let Some(s) = exports.as_str() {
            candidates.push(s.to_string());
        } else if let Some(obj) = exports.as_object() {
            for key in ["import", "require", "."] {
                if let Some(v) = obj.get(key).and_then(|v| v.as_str()) {
                    candidates.push(v.to_string());
                }
            }
        }
    }
    candidates.push("src/index.ts".to_string());
    candidates.push("src/index.tsx".to_string());
    candidates.push("src/index.js".to_string());
    candidates.push("src/index.jsx".to_string());

    for cand in candidates {
        let p = member.join(&cand);
        if p.is_file() {
            return relpath_for(root, &p);
        }
        // `./dist/index.js` style with missing extension
        if !cand.contains('.') {
            for ext in PROBE_EXTS {
                let mut with_ext = cand.clone();
                with_ext.push('.');
                with_ext.push_str(ext);
                let q = member.join(&with_ext);
                if q.is_file() {
                    return relpath_for(root, &q);
                }
            }
        }
    }
    None
}

fn relpath_for(root: &Path, p: &Path) -> Option<String> {
    Some(normalize_rel(
        &p.strip_prefix(root)
            .map(|q| q.to_string_lossy().to_string())
            .unwrap_or_else(|_| p.to_string_lossy().to_string()),
    ))
}

fn inside_root(root: &Path, rel: &str) -> bool {
    let joined = root.join(rel);
    let canonical_joined = joined.canonicalize().unwrap_or(joined);
    let root_canon = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    canonical_joined.starts_with(&root_canon)
}

/// True when `spec` is a filesystem path that points to a directory (used for
/// Python `import module` resolution never resolves to dirs).
pub fn is_absolute_path(spec: &str) -> bool {
    Path::new(spec).is_absolute() || spec.starts_with('/')
}

/// Convert a Python dotted module to an import candidate path.
pub fn python_module_to_rel(module: &str, dots: usize, current_rel: &str) -> Option<String> {
    let parts: Vec<&str> = module.split('.').collect();
    let base = dir_of(current_rel);
    // Walk up once per leading dot, accounting for the current package dir.
    let _ = base;
    let mut up = PathBuf::new();
    // dots == 1 means "current package". dots == n means walk up (n-1) dirs.
    for _ in 0..dots.saturating_sub(1) {
        up.push("..");
    }
    if parts.is_empty() || parts.iter().all(|p| p.is_empty()) {
        return None;
    }
    let mut joined = PathBuf::new();
    for p in &parts {
        if p.is_empty() {
            continue;
        }
        if *p == ".." {
            joined.push("..");
        } else {
            joined.push(p);
        }
    }
    let mut p = PathBuf::new();
    if !current_rel.is_empty() {
        p.push(dir_of(current_rel));
    }
    p.push(up);
    p.push(joined);
    Some(collapse_dots(&normalize_rel(&p.to_string_lossy())))
}

fn collapse_dots(rel: &str) -> String {
    let mut stack: Vec<&str> = Vec::new();
    for seg in rel.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                stack.pop();
            }
            s => stack.push(s),
        }
    }
    stack.join("/")
}

/// Convert a Rust `use` path (already stripped of `use`/`;`/`{group}`) into a
/// list of project-relative probe candidates. Returns an empty vec when the
/// path cannot possibly be local (stdlib or clearly external).
pub fn rust_path_to_candidates(path_body: &str, current_rel: &str) -> Vec<String> {
    let trimmed = path_body.trim();
    let parts: Vec<&str> = trimmed.split("::").filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return Vec::new();
    }
    let first = parts[0];
    let mut cands: Vec<String> = Vec::new();

    let push_path = |cands: &mut Vec<String>, segs: &[&str]| -> usize {
        if segs.is_empty() {
            return 0;
        }
        cands.push(segs.join("/"));
        if segs.len() > 1 {
            // the last segment is often the symbol, not the file
            cands.push(segs[..segs.len() - 1].join("/"));
        }
        segs.len()
    };

    if first == "crate" {
        let rest = &parts[1..];
        push_path(&mut cands, rest);
        // `crate::` resolves relative to the crate root, which is the
        // directory containing lib.rs / main.rs. When that root is a
        // subdirectory (monorepo / nested crate), probe from the current
        // file's own directory as well.
        let base = dir_of(current_rel);
        if !base.is_empty() {
            let prefixed: Vec<String> = cands
                .iter()
                .filter_map(|c| {
                    if c.is_empty() {
                        None
                    } else {
                        Some(format!("{base}/{c}"))
                    }
                })
                .collect();
            cands.extend(prefixed);
        }
    } else if first == "self" {
        let base = dir_of(current_rel);
        let rest = &parts[1..];
        if rest.is_empty() {
            cands.push(base);
            return cands;
        }
        let joined = rest.join("/");
        cands.push(if base.is_empty() {
            joined.clone()
        } else {
            format!("{base}/{joined}")
        });
        if rest.len() > 1 {
            cands.push(if base.is_empty() {
                rest[..rest.len() - 1].join("/")
            } else {
                format!("{base}/{}", rest[..rest.len() - 1].join("/"))
            });
        }
    } else if first == "super" {
        let base_dir = dir_of(current_rel);
        let mut base: Vec<&str> = base_dir.split('/').collect();
        let mut skips = 0;
        let mut cursor = 0;
        while cursor < parts.len() && parts[cursor] == "super" {
            skips += 1;
            cursor += 1;
        }
        let rest = &parts[cursor..];
        if !base.is_empty() && skips > 0 {
            base.truncate(base.len().saturating_sub(skips - 1));
        }
        let b = if base.is_empty() {
            String::new()
        } else {
            base.join("/")
        };
        if rest.is_empty() {
            if !b.is_empty() {
                cands.push(b);
            }
        } else {
            let joined = rest.join("/");
            cands.push(if b.is_empty() {
                joined.clone()
            } else {
                format!("{b}/{joined}")
            });
            if rest.len() > 1 {
                cands.push(if b.is_empty() {
                    rest[..rest.len() - 1].join("/")
                } else {
                    format!("{b}/{}", rest[..rest.len() - 1].join("/"))
                });
            }
        }
    } else if is_external_rust_segment(first) {
        return Vec::new();
    } else {
        // unknown first segment → try both root-relative module and sibling module
        let joined = parts.join("/");
        cands.push(joined.clone());
        if parts.len() > 1 {
            cands.push(parts[..parts.len() - 1].join("/"));
        }
        let base = dir_of(current_rel);
        if !base.is_empty() {
            cands.push(format!("{base}/{joined}"));
        }
    }
    cands
}

pub fn is_external_rust_segment(seg: &str) -> bool {
    matches!(seg, "std" | "core" | "alloc" | "proc_macro" | "test")
}

/// Whether a title-cased Go identifier is exported (public) by Go convention.
pub fn go_exported(name: &str) -> bool {
    name.chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
}

/// Which language id maps to which path probe extension (for `ctx` deps link).
pub fn probe_extensions_for(_lang: LanguageId) -> &'static [&'static str] {
    PROBE_EXTS
}

/// Normalise import spec for `ctx deps` display (strip leading ./ when useful).
pub fn display_spec(spec: &str) -> String {
    spec.to_string()
}

pub fn sanitize_components(rel: &str) -> bool {
    for comp in Path::new(rel).components() {
        if let Component::ParentDir = comp {
            return false;
        }
    }
    true
}

pub fn join_rel(root: &Path, rel: &str) -> PathBuf {
    root.join(rel)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("ctx_res_{nanos}"))
    }

    #[test]
    fn probes_extension_and_index() {
        let root = tmp();
        std::fs::create_dir_all(root.join("lib")).unwrap();
        std::fs::create_dir_all(root.join("components")).unwrap();
        std::fs::write(root.join("lib/util.ts"), "export const x = 1;").unwrap();
        std::fs::write(root.join("components").join("index.js"), "// i").unwrap();

        assert_eq!(
            probe_rel(&root, "", "lib/util").as_deref(),
            Some("lib/util.ts")
        );
        assert_eq!(
            probe_rel(&root, "", "./lib/util").as_deref(),
            Some("lib/util.ts")
        );
        assert_eq!(
            probe_rel(&root, "", "components").as_deref(),
            Some("components/index.js")
        );
        assert_eq!(probe_rel(&root, "", "missing/thing"), None);
    }

    #[test]
    fn probes_dotted_filename_stems() {
        let root = tmp();
        std::fs::create_dir_all(root.join("db/repositories")).unwrap();
        std::fs::write(
            root.join("db/repositories/user.repository.ts"),
            "export const x = 1;",
        )
        .unwrap();
        std::fs::write(root.join("db/auth.routes.ts"), "// routes").unwrap();
        // "user.repository" has a dot, so Path::extension() reports "repository" —
        // the old resolver treated it as already-extensioned and gave up. It must
        // still probe appending source extensions.
        assert_eq!(
            probe_rel(&root, "", "db/repositories/user.repository").as_deref(),
            Some("db/repositories/user.repository.ts")
        );
        assert_eq!(
            probe_rel(&root, "db", "./repositories/user.repository").as_deref(),
            Some("db/repositories/user.repository.ts")
        );
        assert_eq!(
            probe_rel(&root, "", "./db/auth.routes").as_deref(),
            Some("db/auth.routes.ts")
        );
    }

    #[test]
    fn python_module_to_rel_walks_up() {
        assert_eq!(
            python_module_to_rel(".models", 1, "pkg/views.py").as_deref(),
            Some("pkg/models")
        );
        assert_eq!(
            python_module_to_rel("..helpers", 2, "pkg/sub/view.py").as_deref(),
            Some("pkg/helpers")
        );
        assert_eq!(
            python_module_to_rel("src.models", 0, "x.py").as_deref(),
            Some("src/models")
        );
    }

    #[test]
    fn rust_crate_candidates() {
        let c = rust_path_to_candidates("crate::models::User", "src/main.rs");
        assert!(c.iter().any(|p| p == "models/User"));
        assert!(c.iter().any(|p| p == "models"));
        assert!(rust_path_to_candidates("std::collections::HashMap", "x.rs").is_empty());
    }

    #[test]
    fn go_exported_by_case() {
        assert!(go_exported("Hello"));
        assert!(!go_exported("hello"));
    }
}
