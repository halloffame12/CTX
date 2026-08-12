use std::path::Path;

use tree_sitter::{Node, Tree};

use crate::errors::{CtxError, CtxResult};
use crate::lang::LanguageId;
use crate::parser::resolve::{normalize_rel, rust_path_to_candidates};
use crate::parser::traits::{
    Dependency, DependencyType, ParsedFile, ResolvedDependency, Symbol, SymbolKind,
};
use crate::parser::util::{header_signature, make_symbol, node_text, short_text};

pub struct RustParser;

impl RustParser {
    fn language(&self) -> tree_sitter::Language {
        tree_sitter_rust::LANGUAGE.into()
    }

    fn parse_tree(&self, source: &str) -> CtxResult<Tree> {
        let mut parser = tree_sitter::Parser::new();
        let lang = self.language();
        parser
            .set_language(&lang)
            .map_err(|e| CtxError::Other(format!("failed to set Rust language: {e}")))?;
        parser
            .parse(source, None)
            .ok_or_else(|| CtxError::Parse("rust".into(), "no tree produced".into()))
    }

    /// Probe Rust module candidates and return an Internal path when found.
    fn resolve_rust_path(
        &self,
        root: &Path,
        candidates: &[String],
        local_mandatory: bool,
        raw: &str,
    ) -> ResolvedDependency {
        // `crate::graph::database` maps to graph/database.rs at the crate root
        // or under src/. `self`/`super` candidates are relative and carry the
        // current dir already. Probe both bases for the module-root forms.
        let src = root.join("src");
        for base in [root, &src] {
            for c in candidates {
                if c.is_empty() {
                    continue;
                }
                // A trailing symbol segment is stripped by the candidate list,
                // but probe both file and mod.rs forms anyway.
                for cand in [c.clone(), format!("{c}.rs"), format!("{c}/mod.rs")] {
                    let p = base.join(&cand);
                    if p.is_file() {
                        return ResolvedDependency::Internal(rel(root, &p));
                    }
                }
            }
        }
        if local_mandatory {
            ResolvedDependency::Unresolved(raw.to_string())
        } else {
            ResolvedDependency::External(raw.to_string())
        }
    }
}

fn rel(root: &Path, p: &Path) -> String {
    normalize_rel(
        &p.strip_prefix(root)
            .map(|q| q.to_string_lossy().to_string())
            .unwrap_or_else(|_| p.to_string_lossy().to_string()),
    )
}

impl super::traits::LanguageParser for RustParser {
    fn language(&self) -> LanguageId {
        LanguageId::Rust
    }

    fn parse(&self, source: &str, current_rel: &str, root: &Path) -> CtxResult<ParsedFile> {
        let tree = self.parse_tree(source)?;
        let root_node = tree.root_node();
        let has_errors = crate::parser::util::has_errors(&root_node);
        let mut symbols = Vec::new();
        let mut container: Option<String> = None;
        self.collect(&root_node, source, &mut symbols, &mut container);
        let dependencies = self.extract_dependencies(&tree, source, current_rel, root)?;
        Ok(ParsedFile {
            language: LanguageId::Rust,
            symbols,
            dependencies,
            has_errors,
        })
    }

    fn extract_symbols(&self, tree: &Tree, source: &str) -> CtxResult<Vec<Symbol>> {
        let root = tree.root_node();
        let mut out = Vec::new();
        let mut container: Option<String> = None;
        self.collect(&root, source, &mut out, &mut container);
        Ok(out)
    }

    fn extract_dependencies(
        &self,
        tree: &Tree,
        source: &str,
        current_rel: &str,
        root: &Path,
    ) -> CtxResult<Vec<Dependency>> {
        let mut out: Vec<Dependency> = Vec::new();
        let root_node = tree.root_node();
        crate::parser::util::walk(&root_node, &mut |n| {
            if n.kind() != "use_declaration" {
                return;
            }
            let raw = node_text(n, source.as_bytes());
            let body = raw
                .trim_start_matches("use")
                .trim()
                .trim_end_matches(';')
                .trim()
                .to_string();

            // Split off `::{a, b, c}` group when present.
            let (base, group) = match body.find("::{") {
                Some(open) => {
                    let end = body.find('}').unwrap_or(body.len());
                    (
                        body[..open].trim_end().to_string(),
                        body[open + 3..end].to_string(),
                    )
                }
                None => (body.clone(), String::new()),
            };

            let cands = rust_path_to_candidates(&base, current_rel);
            let local_first = base.starts_with("crate::")
                || base.starts_with("self")
                || base.starts_with("super");

            if !group.is_empty() {
                // One dependency per imported name, pointing at the enclosing module.
                for member in group.split(',').map(|m| m.trim()).filter(|m| !m.is_empty()) {
                    let clean = member
                        .split(" as ")
                        .next()
                        .unwrap_or(member)
                        .trim()
                        .to_string();
                    if clean == "*" || clean == "self" {
                        continue;
                    }
                    let resolved = self.resolve_rust_path(root, &cands, local_first, &base);
                    out.push(Dependency {
                        imported_symbol: Some(clean),
                        dependency_type: DependencyType::Use,
                        source_raw: format!("{}::{}", base, member),
                        resolved,
                    });
                }
            } else {
                let imported = use_last_segment(&base);
                let resolved = self.resolve_rust_path(root, &cands, local_first, &raw);
                out.push(Dependency {
                    imported_symbol: imported,
                    dependency_type: DependencyType::Use,
                    source_raw: raw,
                    resolved,
                });
            }
        });
        Ok(out)
    }

    fn skeleton(&self, source: &str) -> CtxResult<String> {
        let lang = self.language();
        Ok(crate::parser::util::skeleton_brace_wrapped(
            source,
            &lang,
            &[
                "function_item",
                "function",
                "closure_expression",
                "async_block",
            ],
            " /* ... implementation hidden ... */ ",
        ))
    }
}

fn use_last_segment(path: &str) -> Option<String> {
    let last = path.rsplit("::").next().map(|s| s.trim()).unwrap_or("");
    if last.is_empty() || last == "*" || last == "self" {
        None
    } else {
        Some(last.to_string())
    }
}

impl RustParser {
    fn collect(
        &self,
        node: &Node,
        source: &str,
        out: &mut Vec<Symbol>,
        container: &mut Option<String>,
    ) {
        match node.kind() {
            "source_file" => {
                for i in 0..node.named_child_count() as u32 {
                    if let Some(c) = node.named_child(i) {
                        self.collect(&c, source, out, container);
                    }
                }
            }
            "function_item" => {
                self.register_function(node, source, out, container, false);
            }
            "struct_item" => {
                if let Some(name) = node.child_by_field_name("name") {
                    let text = node_text(&name, source.as_bytes());
                    out.push(make_symbol(
                        &text,
                        SymbolKind::Struct,
                        node,
                        source.as_bytes(),
                        header_signature(node, source.as_bytes(), body_start(node)),
                        container.clone(),
                        true,
                        visibility_to_opt(node_text(node, source.as_bytes())),
                    ));
                    let prev = container.clone();
                    *container = Some(text);
                    self.collect_fields(node, source, out, container);
                    *container = prev;
                }
            }
            "enum_item" => {
                if let Some(name) = node.child_by_field_name("name") {
                    out.push(make_symbol(
                        &node_text(&name, source.as_bytes()),
                        SymbolKind::Enum,
                        node,
                        source.as_bytes(),
                        header_signature(node, source.as_bytes(), body_start(node)),
                        container.clone(),
                        true,
                        visibility_to_opt(node_text(node, source.as_bytes())),
                    ));
                }
            }
            "trait_item" => {
                if let Some(name) = node.child_by_field_name("name") {
                    let text = node_text(&name, source.as_bytes());
                    out.push(make_symbol(
                        &text,
                        SymbolKind::Trait,
                        node,
                        source.as_bytes(),
                        header_signature(node, source.as_bytes(), body_start(node)),
                        container.clone(),
                        true,
                        visibility_to_opt(node_text(node, source.as_bytes())),
                    ));
                    let prev = container.clone();
                    *container = Some(text);
                    self.collect_scope_items(node, source, out, container, true);
                    *container = prev;
                }
            }
            "impl_item" => {
                let name = impl_name(node, source);
                out.push(make_symbol(
                    &name,
                    SymbolKind::Impl,
                    node,
                    source.as_bytes(),
                    short_text(node, source.as_bytes()),
                    container.clone(),
                    true,
                    None,
                ));
                let prev = container.clone();
                *container = Some(name);
                self.collect_scope_items(node, source, out, container, false);
                *container = prev;
            }
            "const_item" | "static_item" => {
                if let Some(name) = node.child_by_field_name("name") {
                    out.push(make_symbol(
                        &node_text(&name, source.as_bytes()),
                        SymbolKind::Constant,
                        node,
                        source.as_bytes(),
                        short_text(node, source.as_bytes()),
                        container.clone(),
                        true,
                        visibility_to_opt(node_text(node, source.as_bytes())),
                    ));
                }
            }
            "type_item" => {
                if let Some(name) = node.child_by_field_name("name") {
                    out.push(make_symbol(
                        &node_text(&name, source.as_bytes()),
                        SymbolKind::Type,
                        node,
                        source.as_bytes(),
                        short_text(node, source.as_bytes()),
                        container.clone(),
                        true,
                        visibility_to_opt(node_text(node, source.as_bytes())),
                    ));
                }
            }
            "mod_item" => {
                if let Some(name) = node.child_by_field_name("name") {
                    out.push(make_symbol(
                        &node_text(&name, source.as_bytes()),
                        SymbolKind::Module,
                        node,
                        source.as_bytes(),
                        short_text(node, source.as_bytes()),
                        container.clone(),
                        true,
                        visibility_to_opt(node_text(node, source.as_bytes())),
                    ));
                }
            }
            _ => {}
        }
    }

    fn register_function(
        &self,
        node: &Node,
        source: &str,
        out: &mut Vec<Symbol>,
        container: &mut Option<String>,
        trait_method: bool,
    ) {
        let Some(name) = node.child_by_field_name("name") else {
            return;
        };
        let text = node_text(&name, source.as_bytes());
        if text.is_empty() {
            return;
        }
        let kind = if container.is_some() {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        };
        let sig_end = if trait_method && node.kind() == "function_signature_item" {
            node.end_byte()
        } else {
            body_start(node)
        };
        out.push(make_symbol(
            &text,
            kind,
            node,
            source.as_bytes(),
            header_signature(node, source.as_bytes(), sig_end),
            container.clone(),
            true,
            visibility_to_opt(node_text(node, source.as_bytes())),
        ));
    }

    fn collect_scope_items(
        &self,
        node: &Node,
        source: &str,
        out: &mut Vec<Symbol>,
        container: &mut Option<String>,
        trait_method: bool,
    ) {
        // impl/trait bodies nest their items inside a declaration block, so
        // walk all descendants rather than only direct children.
        crate::parser::util::walk(node, &mut |n| match n.kind() {
            "function_item" | "function_signature_item" | "function" => {
                self.register_function(n, source, out, container, trait_method);
            }
            "const_item" | "static_item" => {
                if let Some(name) = n.child_by_field_name("name") {
                    out.push(make_symbol(
                        &node_text(&name, source.as_bytes()),
                        SymbolKind::Constant,
                        n,
                        source.as_bytes(),
                        short_text(n, source.as_bytes()),
                        container.clone(),
                        true,
                        visibility_to_opt(node_text(n, source.as_bytes())),
                    ));
                }
            }
            _ => {}
        });
    }

    fn collect_fields(
        &self,
        node: &Node,
        source: &str,
        out: &mut Vec<Symbol>,
        container: &mut Option<String>,
    ) {
        for i in 0..node.named_child_count() as u32 {
            let Some(c) = node.named_child(i) else {
                continue;
            };
            if matches!(c.kind(), "field_declaration" | "field_initializer")
                && let Some(name) = c.child_by_field_name("name")
            {
                out.push(make_symbol(
                    &node_text(&name, source.as_bytes()),
                    SymbolKind::Field,
                    &c,
                    source.as_bytes(),
                    short_text(&c, source.as_bytes()),
                    container.clone(),
                    true,
                    None,
                ));
            }
        }
    }
}

fn body_start(node: &Node) -> usize {
    node.child_by_field_name("body")
        .map(|b| b.start_byte())
        .unwrap_or_else(|| node.end_byte())
}

fn visibility_to_opt(text: String) -> Option<String> {
    let t = text.trim_start();
    if t.starts_with("pub") {
        Some("public".to_string())
    } else {
        None
    }
}

fn impl_name(node: &Node, source: &str) -> String {
    let text = node_text(node, source.as_bytes());
    let mut t = text.trim();
    for prefix in ["pub ", "unsafe ", "default "] {
        if t.starts_with(prefix) {
            t = t.trim_start_matches(prefix).trim_start();
        }
    }
    let t = t.trim_start_matches("impl").trim();
    let t = if let Some(open) = t.find('{') {
        let s = t[..open].trim();
        s.split(" where ").next().unwrap_or(s)
    } else {
        t
    };
    if let Some(for_idx) = t.rfind(" for ") {
        return t[for_idx + 5..].trim().to_string();
    }
    let name = t.trim();
    if name.is_empty() {
        "impl".to_string()
    } else {
        name.to_string()
    }
}
