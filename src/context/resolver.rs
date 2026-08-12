//! Candidate resolver: given a task, resolve which files/symbols in the graph
//! are worth including. This is the input stage of the context engine.

use std::collections::HashMap;
use std::path::Path;

use crate::context::ranking::score_symbol;
use crate::errors::CtxResult;
use crate::graph::database::{Database, FileRecord};

/// Map from keyword -> candidate files that mention it. Used by the builder
/// as a first filter; the builder scores the result.
pub fn candidates_for_keywords(
    db: &Database,
    root: &Path,
    keywords: &[String],
) -> CtxResult<Vec<Candidate>> {
    let _ = root;
    let data = db.context_load()?;
    let mut seen: HashMap<i64, f64> = HashMap::new();
    for s in &data.symbols {
        let Some(file) = data.files.get(&s.file_id) else {
            continue;
        };
        let score = score_symbol(
            &s.name,
            s.signature.as_deref().unwrap_or(""),
            &file.path,
            keywords,
        );
        if score > 0.0 {
            *seen.entry(s.file_id).or_insert(0.0) += score;
        }
    }
    let mut out: Vec<Candidate> = Vec::new();
    for (id, score) in seen {
        if let Some(f) = data.files.get(&id) {
            out.push(Candidate {
                file: f.clone(),
                score,
            });
        }
    }
    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.file.path.cmp(&b.file.path))
    });
    Ok(out)
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub file: FileRecord,
    pub score: f64,
}
