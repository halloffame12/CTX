use std::path::Path;

use tree_sitter::{Node, Tree};

use crate::errors::{CtxError, CtxResult};
use crate::lang::LanguageId;
use crate::parser::resolve::{probe_rel, python_module_to_rel, resolve_import};
use crate::parser::traits::{
    Dependency, DependencyType, ParsedFile, ResolvedDependency, Symbol, SymbolKind,
};
use crate::parser::util::{header_signature, make_symbol, node_text, short_text};

pub struct PythonParser;

impl PythonParser {
    fn language(&self) -> tree_sitter::Language {
        tree_sitter_python::LANGUAGE.into()
    }

    fn parse_tree(&self, source: &str) -> CtxResult<Tree> {
        let mut parser = tree_sitter::Parser::new();
        let lang = self.language();
        parser
            .set_language(&lang)
            .map_err(|e| CtxError::Other(format!("failed to set Python language: {e}")))?;
        parser
            .parse(source, None)
            .ok_or_else(|| CtxError::Parse("python".into(), "no tree produced".into()))
    }
}

impl super::traits::LanguageParser for PythonParser {
    fn language(&self) -> LanguageId {
        LanguageId::Python
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
            language: LanguageId::Python,
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
        crate::parser::util::walk(&root_node, &mut |n| match n.kind() {
            "import_statement" => {
                for i in 0..n.named_child_count() as u32 {
                    let Some(c) = n.named_child(i) else { continue };
                    if c.kind() == "dotted_name" || c.kind() == "aliased_import" {
                        let spec = node_text(&c, source.as_bytes());
                        let spec_path = spec.replace('.', "/");
                        let resolved = resolve_import(root, current_rel, &spec_path);
                        out.push(Dependency {
                            imported_symbol: None,
                            dependency_type: DependencyType::PyImport,
                            source_raw: spec,
                            resolved,
                        });
                    }
                }
            }
            "import_from_statement" => {
                let (module, dots) = from_module_details(n, source);
                let names = from_names(n, source);
                if module.is_empty() && dots == 0 && names.is_empty() {
                    return;
                }
                let resolved = if dots > 0 {
                    python_relative_resolution(root, current_rel, &module, dots)
                        .unwrap_or_else(|| ResolvedDependency::Unresolved(module.clone()))
                } else if module.is_empty() {
                    ResolvedDependency::Unresolved(module.clone())
                } else {
                    let spec_path = module.replace('.', "/");
                    resolve_import(root, current_rel, &spec_path)
                };
                if names.is_empty() {
                    out.push(Dependency {
                        imported_symbol: None,
                        dependency_type: DependencyType::PyFrom,
                        source_raw: module,
                        resolved,
                    });
                } else {
                    for name in names {
                        out.push(Dependency {
                            imported_symbol: Some(name.clone()),
                            dependency_type: DependencyType::PyFrom,
                            source_raw: module.clone(),
                            resolved: resolved.clone(),
                        });
                    }
                }
            }
            _ => {}
        });
        Ok(out)
    }

    fn skeleton(&self, source: &str, _current_rel: &str) -> CtxResult<String> {
        let tree = self.parse_tree(source)?;
        let root = tree.root_node();
        let mut ranges: Vec<(usize, usize, String)> = Vec::new();
        crate::parser::util::walk(&root, &mut |n| {
            if (n.kind() == "function_definition" || n.kind() == "async_function_definition")
                && let Some(body) = n.child_by_field_name("body")
                && body.kind() == "block"
            {
                let r = body.byte_range();
                if r.end > r.start {
                    let indent = first_line_indent(source, r.start);
                    ranges.push((r.start, r.end, format!("\n{indent}...")));
                }
            }
        });
        Ok(splice_python(source, &ranges))
    }
}

impl PythonParser {
    fn collect(
        &self,
        node: &Node,
        source: &str,
        out: &mut Vec<Symbol>,
        container: &mut Option<String>,
    ) {
        match node.kind() {
            "module" => {
                for i in 0..node.named_child_count() as u32 {
                    if let Some(c) = node.named_child(i) {
                        self.collect(&c, source, out, container);
                    }
                }
            }
            "decorated_definition" => {
                for i in 0..node.named_child_count() as u32 {
                    if let Some(c) = node.named_child(i)
                        && matches!(
                            c.kind(),
                            "class_definition"
                                | "function_definition"
                                | "async_function_definition"
                        )
                    {
                        self.collect(&c, source, out, container);
                    }
                }
            }
            "class_definition" => {
                if let Some(name) = node.child_by_field_name("name") {
                    let text = node_text(&name, source.as_bytes());
                    out.push(make_symbol(
                        &text,
                        SymbolKind::Class,
                        node,
                        source.as_bytes(),
                        header_signature(node, source.as_bytes(), body_start(node)),
                        container.clone(),
                        true,
                        None,
                    ));
                    let prev = container.clone();
                    *container = Some(text);
                    if let Some(body) = node.child_by_field_name("body") {
                        for i in 0..body.named_child_count() as u32 {
                            if let Some(c) = body.named_child(i) {
                                self.collect(&c, source, out, container);
                            }
                        }
                    }
                    *container = prev;
                }
            }
            "function_definition" | "async_function_definition" => {
                if let Some(name) = node.child_by_field_name("name") {
                    let text = node_text(&name, source.as_bytes());
                    let kind = if container.is_some() {
                        SymbolKind::Method
                    } else {
                        SymbolKind::Function
                    };
                    let sig = header_signature(node, source.as_bytes(), body_start(node));
                    out.push(make_symbol(
                        &text,
                        kind,
                        node,
                        source.as_bytes(),
                        sig,
                        container.clone(),
                        !text.starts_with('_'),
                        None,
                    ));
                }
            }
            "assignment" | "augmented_assignment" | "named_expression" => {
                if (container.is_some() || node.parent().map(|p| p.kind()) == Some("module"))
                    && let Some(left) = node.child_by_field_name("left")
                    && left.kind() == "identifier"
                {
                    let text = node_text(&left, source.as_bytes());
                    let kind = if container.is_some() {
                        SymbolKind::Field
                    } else if is_const_name(&text) {
                        SymbolKind::Constant
                    } else {
                        SymbolKind::Variable
                    };
                    out.push(make_symbol(
                        &text,
                        kind,
                        &left_node(node, source),
                        source.as_bytes(),
                        short_text(node, source.as_bytes()),
                        container.clone(),
                        !text.starts_with('_'),
                        None,
                    ));
                }
            }
            _ => {}
        }
    }
}

fn body_start(node: &Node) -> usize {
    node.child_by_field_name("body")
        .map(|b| b.start_byte())
        .unwrap_or_else(|| node.end_byte())
}

fn is_const_name(name: &str) -> bool {
    name.chars().any(|c| c.is_uppercase()) && name.chars().all(|c| !c.is_lowercase())
}

fn left_node<'t>(node: &Node<'t>, _source: &str) -> Node<'t> {
    node.child_by_field_name("left").unwrap_or(*node)
}

fn first_line_indent(source: &str, start: usize) -> String {
    let rest = &source[start..];
    let first_line = rest.split('\n').next().unwrap_or("");
    let indent: String = first_line
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    if indent.is_empty() {
        "    ".to_string()
    } else if indent.len() < 4 {
        // ensure the placeholder is deeper than the enclosing def
        format!("{}{}", " ".repeat(4 - indent.len() + 4), indent)
    } else {
        indent
    }
}

fn splice_python(source: &str, ranges: &[(usize, usize, String)]) -> String {
    let mut kept: Vec<(usize, usize, String)> = ranges.to_vec();
    kept.sort_by_key(|(s, _, _)| *s);
    let mut out = String::with_capacity(source.len());
    let mut pos = 0;
    for (s, e, repl) in &kept {
        if *s < pos {
            continue;
        }
        out.push_str(&source[pos..*s]);
        out.push_str(repl);
        pos = *e;
    }
    out.push_str(&source[pos..]);
    out
}

fn from_module_details(node: &Node, source: &str) -> (String, usize) {
    // try explicit relative_import child first
    if let Some(rel) = node.child_by_field_name("relative_import") {
        let rel_text = node_text(&rel, source.as_bytes());
        let dots = rel_text.chars().take_while(|c| *c == '.').count();
        let module = node
            .child_by_field_name("module_name")
            .map(|m| node_text(&m, source.as_bytes()))
            .unwrap_or_default();
        return (module, dots);
    }
    let module = node
        .child_by_field_name("module_name")
        .map(|m| node_text(&m, source.as_bytes()))
        .unwrap_or_default();
    let dots = module.chars().take_while(|c| *c == '.').count();
    if dots > 0 {
        let stripped: String = module.chars().skip(dots).collect();
        return (stripped, dots);
    }
    (module, 0)
}

fn from_names(node: &Node, source: &str) -> Vec<String> {
    let mut names = Vec::new();
    for i in 0..node.named_child_count() as u32 {
        let Some(c) = node.named_child(i) else {
            continue;
        };
        match c.kind() {
            "import_list" => {
                for i in 0..c.named_child_count() as u32 {
                    if let Some(spec) = c.named_child(i)
                        && (spec.kind() == "dotted_name" || spec.kind() == "aliased_import")
                    {
                        let text = node_text(&spec, source.as_bytes());
                        let name = text.split(" as ").next().unwrap_or(&text).to_string();
                        names.push(name);
                    }
                }
            }
            "wildcard_import" => {
                return Vec::new(); // `import *` → no names
            }
            _ => {}
        }
    }
    names
}

fn python_relative_resolution(
    root: &Path,
    current_rel: &str,
    module: &str,
    dots: usize,
) -> Option<ResolvedDependency> {
    let rel_hint = python_module_to_rel(module, dots, current_rel)?;
    let spec = if rel_hint.starts_with("..") {
        "/".to_string() + rel_hint.trim_start_matches("..")
    } else {
        rel_hint.clone()
    };
    if spec.is_empty() {
        return None;
    }
    let resolved = probe_rel(root, "", &spec)?;
    Some(ResolvedDependency::Internal(resolved))
}
