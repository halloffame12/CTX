//! Deterministic relevance ranking for context selection.

/// Split a raw string into lowercased keywords (alphanumeric runs), dropping
/// common English stop words that carry no semantic weight for code queries.
pub fn tokenize(text: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "the",
        "and",
        "for",
        "with",
        "this",
        "that",
        "from",
        "into",
        "when",
        "how",
        "what",
        "add",
        "adding",
        "implement",
        "implementation",
        "support",
        "using",
        "use",
        "create",
        "create",
        "make",
        "make",
        "do",
        "does",
        "should",
        "need",
        "needs",
        "want",
        "wants",
        "a",
        "an",
        "are",
        "is",
        "be",
        "by",
        "of",
        "to",
        "in",
        "on",
        "at",
        "it",
        "its",
        "our",
        "your",
        "we",
        "you",
        "them",
        "their",
        "back",
        "again",
        "also",
        "just",
        "about",
        "have",
        "has",
        "had",
        "will",
        "would",
        "can",
        "could",
        "new",
        "some",
        "all",
        "any",
        "not",
        "please",
        "code",
        "the",
        "if",
        "then",
        "else",
        "where",
        "there",
        "here",
        "up",
        "down",
        "over",
        "out",
        "under",
        "through",
        "than",
        "then",
        "as",
        "which",
        "while",
        "each",
        "more",
        "most",
        "other",
        "such",
        "only",
        "own",
        "same",
        "so",
        "than",
        "too",
        "very",
    ];
    let lower = text.to_lowercase();
    let mut words = Vec::new();
    let mut cur = String::new();
    for c in lower.chars() {
        if c.is_alphanumeric() {
            cur.push(c);
        } else {
            if !cur.is_empty() {
                words.push(std::mem::take(&mut cur));
            }
        }
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words.retain(|w| w.len() > 1 && !STOP.contains(&w.as_str()));
    words.dedup();
    words
}

/// Score a single (symbol name, signature, path) candidate against keywords.
pub fn score_symbol(name: &str, signature: &str, path: &str, keywords: &[String]) -> f64 {
    let mut score = 0.0;
    let name_lower = name.to_lowercase();
    let sig_lower = signature.to_lowercase();
    let path_lower = path.to_lowercase();
    for k in keywords {
        if name_lower == *k {
            score += 6.0;
        } else if name_lower.contains(k.as_str()) {
            score += 3.0;
        }
        if sig_lower.contains(k.as_str()) {
            score += 1.0;
        }
        if path_lower.contains(k.as_str()) {
            score += 1.0;
        }
    }
    score
}

/// Human-readable, explainable reasons a symbol matched the task keywords.
/// Mirrors [`score_symbol`] so the listed reasons always justify the score.
pub fn symbol_reasons(name: &str, signature: &str, path: &str, keywords: &[String]) -> Vec<String> {
    let name_lower = name.to_lowercase();
    let sig_lower = signature.to_lowercase();
    let path_lower = path.to_lowercase();
    let mut reasons: Vec<String> = Vec::new();
    for k in keywords {
        if name_lower == *k {
            reasons.push(format!("exact symbol match `{name}`"));
        } else if name_lower.contains(k.as_str()) {
            reasons.push(format!("symbol contains keyword `{k}`"));
        }
        if sig_lower.contains(k.as_str()) && !name_lower.contains(k.as_str()) {
            reasons.push(format!("signature mentions `{k}`"));
        }
        if path_lower.contains(k.as_str()) && !name_lower.contains(k.as_str()) {
            reasons.push(format!("path matches keyword `{k}`"));
        }
    }
    reasons.dedup();
    reasons
}

/// Explainable file-level signals beyond symbol matches.
pub fn file_reasons(
    path: &str,
    keywords: &[String],
    recency: bool,
    framework: bool,
    hub: Option<i64>,
    git_recent: bool,
) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    for k in keywords {
        if path.to_lowercase().contains(k.as_str()) {
            reasons.push(format!("path matches keyword `{k}`"));
        }
    }
    if framework {
        reasons.push("framework-relevant directory".to_string());
    }
    if recency {
        reasons.push("modified recently".to_string());
    }
    if hub.is_some_and(|n| n >= 2) {
        reasons.push(format!("imported by {} files (hub)", hub.unwrap()));
    }
    if git_recent {
        reasons.push("modified in working tree".to_string());
    }
    reasons.dedup();
    reasons
}

const FRAMEWORK_DIRS: &[&str] = &[
    "auth",
    "api",
    "models",
    "services",
    "service",
    "repositories",
    "repository",
    "routes",
    "pages",
    "components",
    "controllers",
    "middleware",
    "handlers",
    "views",
    "templates",
    "migrations",
    "schema",
    "database",
    "db",
    "store",
    "plugins",
    "providers",
    "config",
    "types",
    "schemas",
    "dto",
    "domain",
    "core",
    "infra",
    "infrastructure",
    "internal",
    "webhooks",
    "events",
    "jobs",
    "workers",
    "lib",
    "utils",
    "util",
    "helpers",
    "test",
    "tests",
];

pub fn framework_bonus(path: &str) -> f64 {
    let segs = path.split('/').collect::<Vec<_>>();
    let seg_score = segs
        .iter()
        .filter(|s| FRAMEWORK_DIRS.contains(&s.to_ascii_lowercase().as_str()))
        .count() as f64;
    seg_score * 0.5
}

/// Hash-lurk bonus for files that many others import.
pub fn hub_bonus(dependents_count: i64) -> f64 {
    ((dependents_count as f64) + 1.0).ln() * 0.25
}

pub fn path_keyword_bonus(path: &str, keywords: &[String]) -> f64 {
    let lower = path.to_lowercase();
    keywords
        .iter()
        .filter(|k| lower.contains(k.as_str()))
        .count() as f64
}
