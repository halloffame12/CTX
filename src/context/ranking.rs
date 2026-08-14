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
        } else if prefix_match(&name_lower, k.as_str()) {
            score += 2.0;
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

/// Approximate-vocabulary match: "authentication" ~ "authenticate". True when
/// the two strings share a common prefix of at least 4 characters — enough to
/// be a meaningful stem. Only applied to the symbol name, never to keywords
/// that are pure stop-ish noise.
fn prefix_match(a: &str, b: &str) -> bool {
    let common = a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count();
    common >= 4
}

/// Synonym groups: alternate task vocabularies for the same concept. A task
/// keyword that matches any member of a group (by equality or shared stem)
/// activates the whole group, so "login" also matches code that says
/// `authenticate` and "avatar" matches code that says `picture`.
const SYNONYM_GROUPS: &[&[&str]] = &[
    &[
        "login",
        "signin",
        "sign_in",
        "sign-in",
        "authenticate",
        "authentication",
        "auth",
    ],
    &["signup", "sign_up", "sign-up", "register", "registration"],
    &["password", "passwd", "passcode", "credential", "secret"],
    &["email", "mail", "mailbox"],
    &["avatar", "picture", "photo", "image", "thumbnail"],
    &["profile", "account", "userprofile"],
    &["rate", "ratelimit", "rate_limiting", "throttle", "limit"],
    &["tier", "plan", "subscription", "membership", "level"],
    &["billing", "invoice", "charge", "checkout"],
    &["payment", "payments", "checkout", "charge"],
    &["webhook", "callback", "hook", "eventlistener"],
    &["audit", "log", "history", "trail", "activity"],
    &["preferences", "prefs", "settings", "options", "config"],
    &["notification", "alert", "notify", "notifications"],
    &["admin", "administrator", "staff", "permission", "role"],
    &["export", "download", "dump", "serialize", "extract"],
    &["import", "load", "ingest", "parse"],
    &["reset", "regenerate", "reissue", "recover", "revoke"],
    &["verify", "verification", "confirm", "validate", "check"],
    &["metrics", "stats", "analytics", "counts", "report"],
    &["dashboard", "overview", "report", "home"],
    &["upload", "store", "put", "save", "persist"],
    &["session", "token", "jwt", "authtoken"],
    &["user", "account", "member", "person"],
    &["data", "record", "entity", "info", "information"],
    &["config", "configuration", "settings", "options"],
    &["refund", "reversal", "chargeback"],
    &["search", "query", "find", "lookup"],
    &["changed", "modified", "updated", "diff"],
    &["delete", "remove", "drop", "destroy", "purge"],
    &["error", "exception", "failure", "fault"],
    &["test", "tests", "spec", "unittest"],
];

/// Expand task keywords with their synonym groups so matching tolerates
/// different task vocabulary ("login" → also matches `authenticate`). The
/// original keywords are always kept; expansions are deduplicated and capped.
pub fn expand_keywords(keywords: &[String]) -> Vec<String> {
    let mut out: Vec<String> = keywords.to_vec();
    for k in keywords {
        let k_lower = k.to_lowercase();
        for group in SYNONYM_GROUPS {
            let active = group.iter().any(|member| {
                let m = member.to_lowercase();
                m == k_lower
                    || prefix_match(&m, &k_lower)
                    || k_lower.contains(&m)
                    || m.contains(&k_lower)
            });
            if active {
                for member in group.iter() {
                    let m = member.to_string();
                    if !out.contains(&m) {
                        out.push(m);
                    }
                }
            }
        }
    }
    out.truncate(40);
    out
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
        } else if prefix_match(&name_lower, k.as_str()) {
            reasons.push(format!("symbol matches `{k}` by prefix"));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vocabulary_prefix_match_catches_inflections() {
        assert!(prefix_match("authenticatewithpassword", "authentication"));
        assert!(prefix_match("authentication", "authenticatewithpassword"));
        assert!(prefix_match("configuration", "config"));
        assert!(!prefix_match("cat", "catalog"));
        assert!(!prefix_match("auth", "user"));
    }

    #[test]
    fn score_symbol_prefix_matches_symbol_name() {
        let keywords = vec!["authentication".to_string()];
        let score = score_symbol(
            "authenticateWithPassword",
            "",
            "src/auth/auth.ts",
            &keywords,
        );
        assert!(
            score >= 2.0,
            "prefix match should award points, got {score}"
        );
        let reasons = symbol_reasons(
            "authenticateWithPassword",
            "",
            "src/auth/auth.ts",
            &keywords,
        );
        assert!(
            reasons.iter().any(|r| r.contains("by prefix")),
            "reasons should mention prefix match: {reasons:?}"
        );
    }

    #[test]
    fn short_keywords_do_not_prefix_match() {
        assert!(!prefix_match("ab", "abcdef"));
        assert!(!prefix_match("xyz", "xy"));
    }

    #[test]
    fn synonym_expansion_covers_task_vocabulary() {
        let expanded = expand_keywords(&["login".to_string(), "avatar".to_string()]);
        assert!(
            expanded.contains(&"authenticate".to_string()),
            "{expanded:?}"
        );
        assert!(expanded.contains(&"picture".to_string()), "{expanded:?}");
        assert!(expanded.contains(&"login".to_string()), "original kept");
        assert!(expanded.len() <= 40);
    }

    #[test]
    fn synonym_expansion_does_not_add_irrelevant_terms() {
        let expanded = expand_keywords(&["rate".to_string()]);
        assert!(expanded.contains(&"limit".to_string()), "{expanded:?}");
        assert!(!expanded.contains(&"avatar".to_string()), "{expanded:?}");
    }

    #[test]
    fn score_symbol_matches_via_synonym_expansion() {
        let keywords = expand_keywords(&["login".to_string()]);
        let score = score_symbol(
            "authenticateWithPassword",
            "",
            "src/auth/auth.ts",
            &keywords,
        );
        assert!(
            score >= 2.0,
            "login->authenticate should score, got {score}"
        );
    }
}
