use std::path::Path;

use tree_sitter::{Node, Tree};

use crate::errors::{CtxError, CtxResult};
use crate::lang::LanguageId;
use crate::parser::resolve::resolve_import;
use crate::parser::traits::{Dependency, DependencyType, ParsedFile, Symbol, SymbolKind};
use crate::parser::util::{
    header_signature, is_exported, make_symbol, node_text, short_text, skeleton_brace_wrapped,
    visibility_from,
};

/// Shared implementation for TypeScript and JavaScript (their tree-sitter node
/// kinds are a superset/subset of each other).
pub struct JsParser {
    pub id: LanguageId,
}

impl JsParser {
    pub fn new(id: LanguageId) -> Self {
        Self { id }
    }

    pub fn tree_sitter_language(&self) -> tree_sitter::Language {
        match self.id {
            LanguageId::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            LanguageId::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            _ => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        }
    }

    fn parse_tree(&self, source: &str) -> CtxResult<Tree> {
        let mut parser = tree_sitter::Parser::new();
        let lang = self.tree_sitter_language();
        parser
            .set_language(&lang)
            .map_err(|e| CtxError::Other(format!("failed to set TS language: {e}")))?;
        parser.parse(source, None).ok_or_else(|| {
            CtxError::Parse(
                "javascript/typescript".to_string(),
                "no tree produced".into(),
            )
        })
    }
}

impl super::traits::LanguageParser for JsParser {
    fn language(&self) -> LanguageId {
        self.id
    }

    fn parse(&self, source: &str, current_rel: &str, root: &Path) -> CtxResult<ParsedFile> {
        let tree = self.parse_tree(source)?;
        let (root_node, has_errors) = {
            let r = tree.root_node();
            (r, crate::parser::util::has_errors(&r))
        };
        let symbols = self.extract_symbols_root(&root_node, source)?;
        let dependencies = self.extract_dependencies(&tree, source, current_rel, root)?;
        Ok(ParsedFile {
            language: self.id,
            symbols,
            dependencies,
            has_errors,
        })
    }

    fn extract_symbols(&self, tree: &Tree, source: &str) -> CtxResult<Vec<Symbol>> {
        let root = tree.root_node();
        self.extract_symbols_root(&root, source)
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
            match n.kind() {
                "import_statement" => {
                    let Some(source_node) = n.child_by_field_name("source") else {
                        return;
                    };
                    let spec = strip_string(&node_text(&source_node, source.as_bytes()));
                    let names: Vec<String> =
                        if let Some(clause) = n.child_by_field_name("import_clause") {
                            import_names(&clause, source)
                        } else {
                            Vec::new()
                        };
                    push_deps(
                        &mut out,
                        spec,
                        DependencyType::Import,
                        names,
                        current_rel,
                        root,
                    );
                }
                "export_statement" => {
                    if let Some(source_node) = n.child_by_field_name("source") {
                        let spec = strip_string(&node_text(&source_node, source.as_bytes()));
                        push_deps(
                            &mut out,
                            spec,
                            DependencyType::Export,
                            Vec::new(),
                            current_rel,
                            root,
                        );
                    }
                }
                "import_expression" => {
                    // import("...")
                    for i in 0..n.named_child_count() as u32 {
                        if let Some(c) = n.named_child(i)
                            && c.kind() == "string"
                        {
                            let spec = strip_string(&node_text(&c, source.as_bytes()));
                            push_deps(
                                &mut out,
                                spec,
                                DependencyType::Require,
                                Vec::new(),
                                current_rel,
                                root,
                            );
                        }
                    }
                }
                "call_expression" => {
                    let Some(func) = n.child_by_field_name("function") else {
                        return;
                    };
                    if func.kind() == "identifier" || func.kind() == "import" {
                        let name = node_text(&func, source.as_bytes());
                        if name == "require" || name == "import" {
                            for i in 0..n.named_child_count() as u32 {
                                if let Some(c) = n.named_child(i)
                                    && c.kind() == "string"
                                    && (c.start_position().row > func.start_position().row
                                        || c.start_byte() > func.end_byte())
                                {
                                    let spec = strip_string(&node_text(&c, source.as_bytes()));
                                    push_deps(
                                        &mut out,
                                        spec,
                                        DependencyType::Require,
                                        Vec::new(),
                                        current_rel,
                                        root,
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        });
        Ok(out)
    }

    fn skeleton(&self, source: &str) -> CtxResult<String> {
        let lang = self.tree_sitter_language();
        Ok(skeleton_brace_wrapped(
            source,
            &lang,
            &[
                "function_declaration",
                "function_expression",
                "method_definition",
                "constructor",
                "arrow_function",
                "generator_function_declaration",
                "generator_function_expression",
            ],
            " /* ... implementation hidden ... */ ",
        ))
    }
}

impl JsParser {
    fn extract_symbols_root(&self, root: &Node, source: &str) -> CtxResult<Vec<Symbol>> {
        let mut out: Vec<Symbol> = Vec::new();
        let mut container: Option<String> = None;
        self.collect(root, source, &mut out, &mut container);
        Ok(out)
    }

    fn collect(
        &self,
        node: &Node,
        source: &str,
        out: &mut Vec<Symbol>,
        container: &mut Option<String>,
    ) {
        match node.kind() {
            "program" | "module" | "export_statement" | "statement_block"
                if container.is_none() =>
            {
                // descend into top-level containers only (function bodies excluded
                // by not recursing into function-like nodes)
                for i in 0..node.named_child_count() as u32 {
                    if let Some(c) = node.named_child(i) {
                        self.collect(&c, source, out, container);
                    }
                }
            }
            "class_declaration" | "abstract_class_declaration" => {
                if let Some(name) = node.child_by_field_name("name") {
                    let text = node_text(&name, source.as_bytes());
                    let exported = is_exported(node);
                    let sig = header_signature(node, source.as_bytes(), body_start(node));
                    out.push(make_symbol(
                        &text,
                        SymbolKind::Class,
                        node,
                        source.as_bytes(),
                        sig,
                        container.clone(),
                        exported,
                        None,
                    ));
                    let prev = container.clone();
                    *container = Some(text);
                    if let Some(body) = node.child_by_field_name("body") {
                        self.collect_class_body(&body, source, out, container);
                    }
                    *container = prev;
                }
            }
            "interface_declaration" => {
                if let Some(name) = node.child_by_field_name("name") {
                    let text = node_text(&name, source.as_bytes());
                    out.push(make_symbol(
                        &text,
                        SymbolKind::Interface,
                        node,
                        source.as_bytes(),
                        header_signature(node, source.as_bytes(), body_start(node)),
                        container.clone(),
                        is_exported(node),
                        None,
                    ));
                }
            }
            "internal_module" => {
                if let Some(name) = node
                    .child_by_field_name("name")
                    .or_else(|| node.named_child(0))
                {
                    let text = node_text(&name, source.as_bytes());
                    out.push(make_symbol(
                        &text,
                        SymbolKind::Module,
                        node,
                        source.as_bytes(),
                        short_text(node, source.as_bytes()),
                        container.clone(),
                        is_exported(node),
                        None,
                    ));
                }
            }
            "function_declaration"
            | "function_expression"
            | "generator_function_declaration"
            | "generator_function_expression" => {
                self.register_function(node, source, out, container, false);
            }
            "lexical_declaration" | "variable_declaration" => {
                for i in 0..node.named_child_count() as u32 {
                    if let Some(c) = node.named_child(i)
                        && c.kind() == "variable_declarator"
                    {
                        self.register_declarator(&c, source, out, container, node);
                    }
                }
            }
            "type_alias_declaration" => {
                if let Some(name) = node.child_by_field_name("name") {
                    out.push(make_symbol(
                        &node_text(&name, source.as_bytes()),
                        SymbolKind::Type,
                        node,
                        source.as_bytes(),
                        short_text(node, source.as_bytes()),
                        container.clone(),
                        is_exported(node),
                        None,
                    ));
                }
            }
            "enum_declaration" => {
                if let Some(name) = node.child_by_field_name("name") {
                    out.push(make_symbol(
                        &node_text(&name, source.as_bytes()),
                        SymbolKind::Enum,
                        node,
                        source.as_bytes(),
                        header_signature(node, source.as_bytes(), body_start(node)),
                        container.clone(),
                        is_exported(node),
                        None,
                    ));
                }
            }
            _ => {}
        }
    }

    fn collect_class_body(
        &self,
        body: &Node,
        source: &str,
        out: &mut Vec<Symbol>,
        container: &mut Option<String>,
    ) {
        for i in 0..body.named_child_count() as u32 {
            let Some(c) = body.named_child(i) else {
                continue;
            };
            match c.kind() {
                "method_definition" | "generator_function_declaration" => {
                    self.register_function(&c, source, out, container, true);
                }
                "public_field_definition" | "field_definition" => {
                    if let Some(name) = c.child_by_field_name("name") {
                        let text = node_text(&name, source.as_bytes());
                        let header = header_signature(&c, source.as_bytes(), c.end_byte());
                        let vis = visibility_from(&header);
                        let exported = is_exported(&c);
                        out.push(make_symbol(
                            &text,
                            SymbolKind::Field,
                            &c,
                            source.as_bytes(),
                            header,
                            container.clone(),
                            exported,
                            vis,
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    fn register_function(
        &self,
        node: &Node,
        source: &str,
        out: &mut Vec<Symbol>,
        container: &mut Option<String>,
        _in_class: bool,
    ) {
        let Some(name) = node.child_by_field_name("name") else {
            return;
        };
        let text = node_text(&name, source.as_bytes());
        if text.is_empty() {
            return;
        }
        let exported = is_exported(node);
        let sig = header_signature(node, source.as_bytes(), body_start(node));
        let vis = visibility_from(&sig);
        let kind = if container.is_some() {
            SymbolKind::Method
        } else {
            SymbolKind::Function
        };
        out.push(make_symbol(
            &text,
            kind,
            node,
            source.as_bytes(),
            sig,
            container.clone(),
            exported,
            vis,
        ));
    }

    fn register_declarator(
        &self,
        node: &Node,
        source: &str,
        out: &mut Vec<Symbol>,
        container: &mut Option<String>,
        decl: &Node,
    ) {
        let Some(name) = node.child_by_field_name("name") else {
            return;
        };
        let text = node_text(&name, source.as_bytes());
        if text.is_empty() {
            return;
        }
        let value = node.child_by_field_name("value");
        let is_const = text_kind(decl) == "const";
        let kind = match value.map(|v| v.kind()) {
            Some("arrow_function") | Some("function_expression") => SymbolKind::Function,
            Some("class") => SymbolKind::Class,
            _ => {
                if is_const {
                    SymbolKind::Constant
                } else {
                    SymbolKind::Variable
                }
            }
        };
        let exported = is_exported(decl);
        let sig = short_text(node, source.as_bytes());
        out.push(make_symbol(
            &text,
            kind,
            node,
            source.as_bytes(),
            sig,
            container.clone(),
            exported,
            None,
        ));
    }
}

fn text_kind(node: &Node) -> &'static str {
    match node.kind() {
        "lexical_declaration" => {
            let mut found = "let";
            for i in 0..node.child_count() as u32 {
                if let Some(c) = node.child(i)
                    && !c.is_named()
                {
                    found = c.kind();
                    break;
                }
            }
            found
        }
        _ => "var",
    }
}

fn body_start(node: &Node) -> usize {
    node.child_by_field_name("body")
        .map(|b| b.start_byte())
        .unwrap_or_else(|| node.end_byte())
}

fn strip_string(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 && (s.starts_with('"') || s.starts_with('\'')) {
        return s[1..s.len() - 1].to_string();
    }
    if s.starts_with('`') {
        let inner = s.trim_start_matches('`');
        if inner.contains('$') {
            return String::new(); // template with interpolation → unresolvable
        }
    }
    s.to_string()
}

fn import_names(clause: &Node, source: &str) -> Vec<String> {
    let mut names = Vec::new();
    for i in 0..clause.named_child_count() as u32 {
        let Some(c) = clause.named_child(i) else {
            continue;
        };
        match c.kind() {
            "named_imports" => {
                for i in 0..c.named_child_count() as u32 {
                    if let Some(spec) = c.named_child(i)
                        && spec.kind() == "import_specifier"
                        && let Some(n) = spec.child_by_field_name("name")
                    {
                        names.push(node_text(&n, source.as_bytes()));
                    }
                }
            }
            "namespace_import" => names.push("*".to_string()),
            "identifier" => names.push(node_text(&c, source.as_bytes())),
            _ => {}
        }
    }
    names
}

fn push_deps(
    out: &mut Vec<Dependency>,
    spec: String,
    dep_type: DependencyType,
    names: Vec<String>,
    current_rel: &str,
    root: &Path,
) {
    if spec.is_empty() {
        return;
    }
    let resolved = resolve_import(root, current_rel, &spec);
    if names.is_empty() {
        out.push(Dependency {
            imported_symbol: None,
            dependency_type: dep_type,
            source_raw: spec,
            resolved,
        });
    } else {
        for n in names {
            out.push(Dependency {
                imported_symbol: Some(n.clone()),
                dependency_type: dep_type,
                source_raw: spec.clone(),
                resolved: resolved.clone(),
            });
        }
    }
}
