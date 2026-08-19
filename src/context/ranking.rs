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

/// Split an identifier or path into lowercase word tokens, breaking on
/// non-alphanumeric characters, camelCase boundaries, and letter→digit
/// boundaries ("WarehouseResult1004" → ["warehouse","result","1004"]).
pub fn word_tokens(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut cur = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_alphanumeric() {
            if !cur.is_empty() {
                tokens.push(cur.to_lowercase());
                cur.clear();
            }
            continue;
        }
        let camel = c.is_ascii_uppercase()
            && i > 0
            && (chars[i - 1].is_ascii_lowercase() || chars[i - 1].is_ascii_digit());
        let digit = c.is_ascii_digit() && i > 0 && chars[i - 1].is_ascii_lowercase();
        if (camel || digit) && !cur.is_empty() {
            tokens.push(cur.to_lowercase());
            cur.clear();
        }
        cur.push(c);
    }
    if !cur.is_empty() {
        tokens.push(cur.to_lowercase());
    }
    tokens
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) enum TokenMatch {
    None,
    Prefix,
    Extended,
    Exact,
}

/// True when two words share a meaningful stem ("authenticate" and
/// "authentication" share "authenticat"; "login" and "logistics" share only
/// the 4-letter run "logi", which is NOT enough). Requires a shared prefix of
/// at least 5 characters — long enough to be a real inflection, short enough
/// to tolerate prefix extension like config ↔ configuration.
fn stem_match(a: &str, b: &str) -> bool {
    let common = a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count();
    common >= 5
}

/// Strongest way a keyword hits a set of word tokens. An exact word match
/// beats a substring-at-a-word-edge match ("reporting" for "report"), which
/// beats a shared-stem match ("authenticate" for "authentication"). A keyword
/// is deliberately never matched against the middle of a concatenated
/// identifier ("user" must not match "warehouseResult"), so CamelCase noise
/// like `WarehouseResult1004` can never flood a package for a "user" task.
pub(crate) fn best_token_match(tokens: &[String], k: &str) -> TokenMatch {
    let mut best = TokenMatch::None;
    for t in tokens {
        if t == k {
            return TokenMatch::Exact;
        }
        let at_edge = (t.starts_with(k) || t.ends_with(k)) && k.len() >= 4 && t.len() > k.len();
        let shared_stem = stem_match(t, k);
        best = if at_edge {
            TokenMatch::Extended
        } else if shared_stem {
            best.max(TokenMatch::Prefix)
        } else {
            best
        };
    }
    best
}

/// Score a single (symbol name, signature, path) candidate against keywords.
pub fn score_symbol(name: &str, signature: &str, path: &str, keywords: &[String]) -> f64 {
    let ones: Vec<f64> = vec![1.0; keywords.len()];
    score_symbol_w(name, signature, path, keywords, &ones)
}

/// Weighted variant of [`score_symbol`]. `weights` runs parallel to `keywords`
/// and scales every contribution of that keyword, so a keyword that matches a
/// large fraction of the corpus (e.g. a structural term like "api" in a repo
/// with hundreds of `*.api.ts` modules) contributes proportionally less.
pub fn score_symbol_w(
    name: &str,
    signature: &str,
    path: &str,
    keywords: &[String],
    weights: &[f64],
) -> f64 {
    let name_tokens = word_tokens(name);
    let sig_tokens = word_tokens(signature);
    let path_tokens = word_tokens(path);
    let mut score = 0.0;
    for (i, k) in keywords.iter().enumerate() {
        let w = weights.get(i).copied().unwrap_or(1.0);
        score += match best_token_match(&name_tokens, k) {
            TokenMatch::Exact => 6.0,
            TokenMatch::Extended => 3.0,
            TokenMatch::Prefix => 2.0,
            TokenMatch::None => 0.0,
        } * w;
        if best_token_match(&sig_tokens, k) != TokenMatch::None {
            score += 1.0 * w;
        }
        if best_token_match(&path_tokens, k) != TokenMatch::None {
            score += 1.0 * w;
        }
    }
    score
}

/// Approximate-vocabulary match: "authentication" ~ "authenticate". True when
/// the two strings share a common prefix of at least 4 characters — enough to
/// be a meaningful stem. Only applied to the symbol name, never to keywords
/// that are pure stop-ish noise.
#[cfg(test)]
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
        let k_tokens = word_tokens(&k_lower);
        for group in SYNONYM_GROUPS {
            let active = group.iter().any(|member| {
                let m = member.to_lowercase();
                let m_tokens = word_tokens(&m);
                m == k_lower
                    || best_token_match(&m_tokens, &k_lower) != TokenMatch::None
                    || best_token_match(&k_tokens, member) != TokenMatch::None
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

/// IDF-style per-keyword damping weights. A keyword that matches a large
/// fraction of the corpus carries little signal: in a repo with hundreds of
/// `*.api.ts` modules, the term "api" matches nearly every file, so its exact
/// symbol/name/path hits should contribute proportionally less than a rare
/// term like "auth" that matches a handful of files. `document_frequencies[i]`
/// is the number of distinct files keyword `i` matches (documents containing
/// the term). Returns a weight per keyword in [0, 1]; terms matching at most
/// `RARE` files keep full weight, then the weight falls off linearly so a
/// generic term matching dozens of files is damped hard.
pub fn idf_keyword_weights(document_frequencies: &[usize]) -> Vec<f64> {
    const RARE: f64 = 8.0;
    document_frequencies
        .iter()
        .map(|&df| (RARE / df.max(1) as f64).min(1.0))
        .collect()
}

/// Human-readable, explainable reasons a symbol matched the task keywords.
/// Mirrors [`score_symbol`] so the listed reasons always justify the score.
pub fn symbol_reasons(name: &str, signature: &str, path: &str, keywords: &[String]) -> Vec<String> {
    let name_tokens = word_tokens(name);
    let sig_tokens = word_tokens(signature);
    let path_tokens = word_tokens(path);
    let mut reasons: Vec<String> = Vec::new();
    for k in keywords {
        match best_token_match(&name_tokens, k) {
            TokenMatch::Exact => reasons.push(format!("exact symbol match `{name}`")),
            TokenMatch::Extended => reasons.push(format!("symbol contains keyword `{k}`")),
            TokenMatch::Prefix => reasons.push(format!("symbol matches `{k}` by prefix")),
            TokenMatch::None => {}
        }
        if best_token_match(&sig_tokens, k) != TokenMatch::None
            && best_token_match(&name_tokens, k) == TokenMatch::None
        {
            reasons.push(format!("signature mentions `{k}`"));
        }
        if best_token_match(&path_tokens, k) != TokenMatch::None
            && best_token_match(&name_tokens, k) == TokenMatch::None
        {
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
    let path_tokens = word_tokens(path);
    for k in keywords {
        if best_token_match(&path_tokens, k) != TokenMatch::None {
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
    let ones: Vec<f64> = vec![1.0; keywords.len()];
    path_keyword_bonus_w(path, keywords, &ones)
}

/// Weighted variant of [`path_keyword_bonus`], see [`score_symbol_w`].
pub fn path_keyword_bonus_w(path: &str, keywords: &[String], weights: &[f64]) -> f64 {
    let tokens = word_tokens(path);
    keywords
        .iter()
        .enumerate()
        .filter(|(_, k)| best_token_match(&tokens, k) != TokenMatch::None)
        .map(|(i, _)| weights.get(i).copied().unwrap_or(1.0))
        .sum()
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
    fn stem_match_rejects_short_shared_runs() {
        // "login" and "logistics" share only the 4-letter run "logi".
        assert!(!stem_match("login", "logistics"));
        assert!(!stem_match("logistics", "login"));
        // real inflections survive
        assert!(stem_match("authenticate", "authentication"));
        assert!(stem_match("config", "configuration"));
        assert!(stem_match("verification", "verify"));
        // keyword "login" must not pull a logistics filler symbol into a package
        let tokens = vec![
            "logistics".to_string(),
            "result".to_string(),
            "1004".to_string(),
        ];
        assert_eq!(best_token_match(&tokens, "login"), TokenMatch::None);
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
