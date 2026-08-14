use std::path::Path;

use tree_sitter::{Node, Tree};

use crate::errors::{CtxError, CtxResult};
use crate::lang::LanguageId;
use crate::parser::resolve::resolve_import;
use crate::parser::traits::{
    Dependency, DependencyType, ParsedFile, ResolvedDependency, Symbol, SymbolKind,
};
use crate::parser::util::{header_signature, make_symbol, node_text, short_text};

pub struct GoParser {
    /// Value of `module` in go.mod at index time; None when absent.
    pub module_path: Option<String>,
}

impl Default for GoParser {
    fn default() -> Self {
        Self::new()
    }
}

impl GoParser {
    pub fn new() -> Self {
        Self { module_path: None }
    }

    pub fn detect(root: &Path) -> Self {
        let module_path = read_go_module(root);
        Self { module_path }
    }

    fn language(&self) -> tree_sitter::Language {
        tree_sitter_go::LANGUAGE.into()
    }

    fn parse_tree(&self, source: &str) -> CtxResult<Tree> {
        let mut parser = tree_sitter::Parser::new();
        let lang = self.language();
        parser
            .set_language(&lang)
            .map_err(|e| CtxError::Other(format!("failed to set Go language: {e}")))?;
        parser
            .parse(source, None)
            .ok_or_else(|| CtxError::Parse("go".into(), "no tree produced".into()))
    }

    fn resolve_go_import(&self, root: &Path, path: &str) -> ResolvedDependency {
        let path = path.trim();
        if path.is_empty() {
            return ResolvedDependency::Unresolved(String::new());
        }
        if path.starts_with("./") || path.starts_with("../") {
            return resolve_import(root, "", path);
        }
        if let Some(module) = &self.module_path {
            if path == module.as_str() {
                // importing the package that lives at module root
                if let Some(rel) = go_internal_file(root, "") {
                    return ResolvedDependency::Internal(rel);
                }
            }
            if let Some(rest) = path.strip_prefix(&format!("{module}/")) {
                if let Some(rel) = go_internal_file(root, rest) {
                    return ResolvedDependency::Internal(rel);
                }
                return ResolvedDependency::Unresolved(path.to_string());
            }
        }
        ResolvedDependency::External(path.to_string())
    }
}

/// Map an import path suffix to a concrete .go file in the project.
fn go_internal_file(root: &Path, rest: &str) -> Option<String> {
    if rest.is_empty() {
        return None;
    }
    let mut cands: Vec<String> = Vec::new();
    if let Some((dir, file)) = rest.rsplit_once('/') {
        cands.push(format!("{dir}/{file}.go"));
        cands.push(rest.to_string());
    } else {
        cands.push(format!("{rest}.go"));
        cands.push(rest.to_string());
    }
    for c in &cands {
        let p = root.join(c);
        if p.is_file() {
            return Some(crate::parser::resolve::normalize_rel(c));
        }
    }
    // the path may point to a package directory
    let dir = root.join(rest);
    if dir.is_dir()
        && let Ok(read) = std::fs::read_dir(&dir)
    {
        for entry in read.flatten() {
            let p = entry.path();
            if p.extension().map(|e| e == "go").unwrap_or(false) {
                let rel = p
                    .strip_prefix(root)
                    .map(|q| q.to_string_lossy().to_string())
                    .unwrap_or_default();
                return Some(crate::parser::resolve::normalize_rel(&rel));
            }
        }
    }
    None
}

fn read_go_module(root: &Path) -> Option<String> {
    let content = std::fs::read_to_string(root.join("go.mod")).ok()?;
    content.lines().find_map(|l| {
        let l = l.trim();
        l.strip_prefix("module ").map(|m| m.trim().to_string())
    })
}

impl super::traits::LanguageParser for GoParser {
    fn language(&self) -> LanguageId {
        LanguageId::Go
    }

    fn parse(&self, source: &str, current_rel: &str, root: &Path) -> CtxResult<ParsedFile> {
        let tree = self.parse_tree(source)?;
        let root_node = tree.root_node();
        let has_errors = crate::parser::util::has_errors(&root_node);
        let mut symbols = Vec::new();
        self.collect(&root_node, source, &mut symbols);
        let dependencies = self.extract_dependencies(&tree, source, current_rel, root)?;
        Ok(ParsedFile {
            language: LanguageId::Go,
            symbols,
            dependencies,
            has_errors,
        })
    }

    fn extract_symbols(&self, tree: &Tree, source: &str) -> CtxResult<Vec<Symbol>> {
        let root = tree.root_node();
        let mut out = Vec::new();
        self.collect(&root, source, &mut out);
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
            if n.kind() != "import_spec" {
                return;
            }
            let Some(path_node) = n.named_child(0) else {
                return;
            };
            let path = node_text(&path_node, source.as_bytes())
                .trim_matches('"')
                .to_string();
            if path.is_empty() || path == "C" {
                return;
            }
            let resolved = self.resolve_go_import(root, &path);
            out.push(Dependency {
                imported_symbol: None,
                dependency_type: DependencyType::GoImport,
                source_raw: path.clone(),
                resolved,
            });
        });
        let _ = current_rel;
        Ok(out)
    }

    fn skeleton(&self, source: &str, _current_rel: &str) -> CtxResult<String> {
        let lang = self.language();
        Ok(crate::parser::util::skeleton_brace_wrapped(
            source,
            &lang,
            &["function_declaration", "method_declaration", "func_literal"],
            " /* ... implementation hidden ... */ ",
        ))
    }
}

impl GoParser {
    fn collect(&self, node: &Node, source: &str, out: &mut Vec<Symbol>) {
        match node.kind() {
            "source_file" => {
                for i in 0..node.named_child_count() as u32 {
                    if let Some(c) = node.named_child(i) {
                        self.collect(&c, source, out);
                    }
                }
            }
            "function_declaration" => {
                if let Some(name) = node.child_by_field_name("name") {
                    let text = node_text(&name, source.as_bytes());
                    out.push(make_symbol(
                        &text,
                        SymbolKind::Function,
                        node,
                        source.as_bytes(),
                        signature_text(node, source),
                        None,
                        crate::parser::resolve::go_exported(&text),
                        None,
                    ));
                }
            }
            "method_declaration" => {
                if let Some(name) = node.child_by_field_name("name") {
                    let text = node_text(&name, source.as_bytes());
                    out.push(make_symbol(
                        &text,
                        SymbolKind::Method,
                        node,
                        source.as_bytes(),
                        signature_text(node, source),
                        None,
                        crate::parser::resolve::go_exported(&text),
                        None,
                    ));
                }
            }
            "type_declaration" => {
                for i in 0..node.named_child_count() as u32 {
                    if let Some(c) = node.named_child(i)
                        && c.kind() == "type_spec"
                        && let Some(name) = c.child_by_field_name("name")
                    {
                        let text = node_text(&name, source.as_bytes());
                        let kind = match c.child_by_field_name("type").map(|t| t.kind()) {
                            Some("struct_type") => SymbolKind::Struct,
                            Some("interface_type") => SymbolKind::Interface,
                            _ => SymbolKind::Type,
                        };
                        out.push(make_symbol(
                            &text,
                            kind,
                            &c,
                            source.as_bytes(),
                            short_text(&c, source.as_bytes()),
                            None,
                            crate::parser::resolve::go_exported(&text),
                            None,
                        ));
                    }
                }
            }
            "const_declaration" | "var_declaration" => {
                let is_const = node.kind() == "const_declaration";
                self.collect_specs(node, source, out, is_const);
            }
            _ => {}
        }
    }

    fn collect_specs(&self, node: &Node, source: &str, out: &mut Vec<Symbol>, is_const: bool) {
        for i in 0..node.named_child_count() as u32 {
            let Some(c) = node.named_child(i) else {
                continue;
            };
            if c.kind() != "const_spec" && c.kind() != "var_spec" {
                continue;
            }
            for i in 0..c.named_child_count() as u32 {
                let Some(name) = c.named_child(i) else {
                    continue;
                };
                if name.kind() == "type_identifier" || name.kind() == "identifier" {
                    let text = node_text(&name, source.as_bytes());
                    out.push(make_symbol(
                        &text,
                        if is_const {
                            SymbolKind::Constant
                        } else {
                            SymbolKind::Variable
                        },
                        &c,
                        source.as_bytes(),
                        short_text(&c, source.as_bytes()),
                        None,
                        crate::parser::resolve::go_exported(&text),
                        None,
                    ));
                }
            }
        }
    }
}

fn signature_text(node: &Node, source: &str) -> String {
    let end = node
        .child_by_field_name("body")
        .map(|b| b.start_byte())
        .unwrap_or_else(|| node.end_byte());
    header_signature(node, source.as_bytes(), end)
}
