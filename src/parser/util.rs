use crate::errors::CtxResult;
use crate::lang::Span;
use crate::parser::traits::{Symbol, SymbolKind};
use tree_sitter::{Node, Tree};

pub const MAX_SIGNATURE: usize = 240;

pub fn node_text(node: &Node, source: &[u8]) -> String {
    node.utf8_text(source)
        .map(|s| s.to_string())
        .unwrap_or_default()
}

pub fn span_of(node: &Node) -> Span {
    let range = node.byte_range();
    Span {
        start_byte: range.start,
        end_byte: range.end,
        start_line: node.start_position().row as u32 + 1,
        end_line: node.end_position().row as u32 + 1,
    }
}

/// Collapse every run of whitespace into single spaces; useful for signatures.
pub fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut ws = false;
    for c in s.chars() {
        if c.is_whitespace() {
            ws = true;
        } else {
            if ws && !out.is_empty() {
                out.push(' ');
            }
            out.push(c);
            ws = false;
        }
    }
    out
}

/// Signature of a declaration node truncated at `end` bytes (typically the
/// start of its body) and normalised onto a single line.
pub fn header_signature(node: &Node, source: &[u8], end: usize) -> String {
    let text = node.utf8_text(source).unwrap_or_default();
    let keep = text.get(..end).unwrap_or(text);
    let mut sig = collapse_ws(keep.trim());
    if sig.chars().count() > MAX_SIGNATURE {
        sig = sig.chars().take(MAX_SIGNATURE).collect::<String>();
        sig.push_str("...");
    }
    sig
}

pub fn short_text(node: &Node, source: &[u8]) -> String {
    let text = collapse_ws(node.utf8_text(source).unwrap_or_default().trim());
    if text.chars().count() > MAX_SIGNATURE {
        let mut t: String = text.chars().take(MAX_SIGNATURE).collect();
        t.push_str("...");
        return t;
    }
    text
}

pub fn is_exported(node: &Node) -> bool {
    matches!(node.parent().map(|p| p.kind()), Some("export_statement"))
}

#[allow(clippy::too_many_arguments)]
pub fn make_symbol(
    name: &str,
    kind: SymbolKind,
    node: &Node,
    _source: &[u8],
    signature: String,
    parent: Option<String>,
    exported: bool,
    visibility: Option<String>,
) -> Symbol {
    Symbol {
        name: name.to_string(),
        kind,
        signature,
        span: span_of(node),
        parent,
        visibility,
        exported,
    }
}

/// Detect visibility keywords in a declaration header.
pub fn visibility_from(text: &str) -> Option<String> {
    let t = text.trim_start();
    if t.starts_with("private ") || t == "private" {
        return Some("private".to_string());
    }
    if t.starts_with("protected") {
        return Some("protected".to_string());
    }
    if t.starts_with("public") {
        return Some("public".to_string());
    }
    None
}

/// Depth-first walk over a node and all of its descendants.
pub fn walk(node: &Node, f: &mut impl FnMut(&Node)) {
    f(node);
    for i in 0..node.child_count() as u32 {
        if let Some(c) = node.child(i) {
            walk(&c, f);
        }
    }
}

pub fn has_errors(root: &Node) -> bool {
    let mut found = false;
    walk(root, &mut |n| {
        if n.is_error() || n.is_missing() {
            found = true;
        }
    });
    found
}

pub fn ensure_parse<'t>(tree: &'t Tree, _source: &'t [u8]) -> CtxResult<(Node<'t>, bool)> {
    let root = tree.root_node();
    Ok((root, has_errors(&root)))
}

/// Parse a source string with the given tree-sitter language.
pub fn parse_with(lang: &tree_sitter::Language, source: &str) -> Option<Tree> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(lang).ok()?;
    parser.parse(source, None)
}

/// Replace interiors of brace-wrapped body nodes for function-like kinds.
///
/// The result keeps braces and every line of surrounding structure, collapsing
/// each implementation body into a single placeholder comment. Always yields
/// syntactically valid code for brace languages.
pub fn skeleton_brace_wrapped(
    source: &str,
    lang: &tree_sitter::Language,
    functionish_kinds: &[&str],
    placeholder: &str,
) -> String {
    let Some(tree) = parse_with(lang, source) else {
        return source.to_string();
    };
    let root = tree.root_node();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    walk(&root, &mut |n| {
        if functionish_kinds.contains(&n.kind())
            && let Some(body) = n.child_by_field_name("body")
            && matches!(body.kind(), "statement_block" | "block")
        {
            let r = body.byte_range();
            if r.end > r.start + 2 {
                ranges.push((r.start + 1, r.end - 1));
            }
        }
    });
    splice_ranges(source, &ranges, placeholder)
}

/// Splice `placeholder` into each (byte) range, dropping any range nested
/// inside another.
pub fn splice_ranges(source: &str, ranges: &[(usize, usize)], placeholder: &str) -> String {
    let mut rr: Vec<(usize, usize)> = ranges.to_vec();
    rr.sort_by_key(|(s, _)| *s);
    let mut kept: Vec<(usize, usize)> = Vec::new();
    for &(s, e) in &rr {
        if kept.iter().any(|&(ks, ke)| s >= ks && e <= ke) {
            continue;
        }
        kept.push((s, e));
    }
    let mut out = String::with_capacity(source.len());
    let mut pos = 0;
    for &(s, e) in &kept {
        if s < pos {
            continue;
        }
        out.push_str(&source[pos..s]);
        out.push_str(placeholder);
        pos = e;
    }
    out.push_str(&source[pos..]);
    out
}
