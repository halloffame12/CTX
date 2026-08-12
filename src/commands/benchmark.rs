use std::time::Instant;

use crate::commands::Project;
use crate::errors::CtxResult;
use crate::graph::database::Database;
use crate::output::{Default, Term, emit_json};

pub fn cmd_benchmark(project: &Project, t: &Term) -> CtxResult<()> {
    let samples = 3;
    let mut rows: Vec<(String, u64)> = Vec::new();

    let total = Instant::now();

    let mut db_open = Vec::new();
    for _ in 0..samples {
        let s = Instant::now();
        let d = Database::open(&project.root)?;
        drop(d);
        db_open.push(s.elapsed().as_millis() as u64);
    }
    rows.push(("database open".to_string(), median(&db_open)));

    // fresh reindex timing (incremental pass over cache)
    let now = Instant::now();
    let mut db = Database::open(&project.root)?;
    let mut file_count = 0u64;
    for f in db.all_files()? {
        let rel = f.path.clone();
        let full = project.root.join(&rel);
        if full.exists()
            && let Some(lang) = crate::lang::language_of_path(&full)
        {
            file_count += 1;
            let _ = crate::indexing::incremental::index_single_file(
                &project.root,
                &mut db,
                &rel,
                lang,
                &project.config,
            );
        }
    }
    let idx = now.elapsed();
    rows.push((
        format!("indexed {file_count} files"),
        idx.as_millis() as u64,
    ));

    let now = Instant::now();
    let _ = db.stats()?;
    rows.push(("stats query".to_string(), now.elapsed().as_millis() as u64));

    let now = Instant::now();
    let _ = db.search("main", None, 10)?;
    rows.push((
        "search (main, 10)".to_string(),
        now.elapsed().as_millis() as u64,
    ));

    let now = Instant::now();
    let _ = crate::context::build_context(
        &db,
        &project.root,
        "how is the parser structured",
        &project.config,
        false,
    );
    rows.push((
        "context build".to_string(),
        now.elapsed().as_millis() as u64,
    ));

    let now = Instant::now();
    for f in db.all_files()?.into_iter().take(5) {
        let full = project.root.join(&f.path);
        if full.exists()
            && let Some(lang) = crate::lang::language_of_path(&full)
            && let Ok(src) = std::fs::read_to_string(&full)
        {
            let _ = crate::context::skeleton::skeleton_for(&project.root, &f.path, lang, &src);
        }
    }
    rows.push(("skeleton x5".to_string(), now.elapsed().as_millis() as u64));

    drop(db);
    let wall = total.elapsed().as_millis() as u64;

    let value = serde_json::json!({
        "samples": samples,
        "wall_ms": wall,
        "results": rows.iter().map(|(n, v)| serde_json::json!({"name": n, "median_ms": v})).collect::<Vec<_>>(),
    });

    if t.is_json() {
        emit_json(&value);
        return Ok(());
    }

    println!("ctx benchmarks (median over {samples} runs)");
    println!();
    for (name, ms) in &rows {
        println!(
            "  {:<28} {}",
            name,
            t.style(Default::CYAN, &format!("{ms} ms"))
        );
    }
    println!();
    println!(
        "  total wall: {} ms",
        t.style(Default::BOLD, &format!("{wall}"))
    );
    Ok(())
}

fn median(v: &[u64]) -> u64 {
    let mut v = v.to_vec();
    v.sort_unstable();
    if v.is_empty() {
        return 0;
    }
    v[v.len() / 2]
}
