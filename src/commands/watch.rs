use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use notify::{EventHandler, RecursiveMode, Result as NotifyResult, Watcher};

use crate::commands::Project;
use crate::errors::CtxResult;
use crate::graph::database::{Database, FileRecord};
use crate::indexing::scanner::rel_path;
use crate::output::{Default, Term};

struct Sink(mpsc::Sender<NotifyResult<notify::Event>>);

impl EventHandler for Sink {
    fn handle_event(&mut self, event: NotifyResult<notify::Event>) {
        let _ = self.0.send(event);
    }
}

pub fn cmd_watch(project: &Project, t: &Term) -> CtxResult<()> {
    if !project.config.watch.enabled {
        t.e("watch is disabled in .ctx/config.toml ([watch].enabled = true)");
        return Ok(());
    }
    let (tx, rx) = mpsc::channel::<NotifyResult<notify::Event>>();
    let mut watcher = notify::recommended_watcher(Sink(tx))
        .map_err(|e| crate::errors::CtxError::Other(format!("failed to start watcher: {e}")))?;
    watcher
        .watch(&project.root, RecursiveMode::Recursive)
        .map_err(|e| crate::errors::CtxError::Other(format!("watch: {e}")))?;

    if !t.is_json() {
        t.p(&format!(
            "{} watching {} (Ctrl+C to stop)",
            t.style(Default::GREEN, "ctx watch"),
            project.root.display()
        ));
    }

    let debounce = Duration::from_millis(project.config.watch.debounce_ms);
    let mut pending: Vec<std::path::PathBuf> = Vec::new();
    let mut last_flush = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(event)) => {
                for p in event.paths {
                    pending.push(p);
                }
                if last_flush.elapsed() >= debounce && !pending.is_empty() {
                    flush(project, &mut pending, t);
                    last_flush = Instant::now();
                }
            }
            Ok(Err(e)) => {
                t.e(&format!("watch error: {e}"));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if last_flush.elapsed() >= debounce && !pending.is_empty() {
                    flush(project, &mut pending, t);
                    last_flush = Instant::now();
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

fn flush(project: &Project, pending: &mut Vec<std::path::PathBuf>, t: &Term) {
    let mut paths: Vec<std::path::PathBuf> = std::mem::take(pending);
    paths.sort();
    paths.dedup();
    let mut db = match Database::open(&project.root) {
        Ok(d) => d,
        Err(_) => return,
    };
    for path in paths {
        let rel = rel_path(&project.root, &path);
        if rel.starts_with(".ctx/") {
            continue;
        }
        handle_change(project, &mut db, &rel, t);
    }
}

fn emit_watch_json(value: &serde_json::Value) {
    println!("{}", serde_json::to_string(value).unwrap_or_default());
}

fn handle_change(project: &Project, db: &mut Database, rel: &str, t: &Term) {
    let full = project.root.join(rel);
    if full.is_dir() {
        // directory-level events are noisy; ignore (files carry the change)
        return;
    }
    if !full.exists() {
        if let Ok(Some(_)) = db.file_by_path(rel) {
            let _ = crate::indexing::incremental::remove_file(&project.root, db, rel);
            if t.is_json() {
                emit_watch_json(&serde_json::json!({ "event": "deleted", "path": rel }));
            } else {
                t.p(&format!("{} deleted: {rel}", t.style(Default::RED, "×")));
            }
        }
        return;
    }
    let Some(lang) = crate::lang::language_of_path(&full) else {
        return;
    };
    let before: Option<FileRecord> = db.file_by_path(rel).ok().flatten();
    match crate::indexing::incremental::index_single_file(
        &project.root,
        db,
        rel,
        lang,
        &project.config,
    ) {
        Ok(()) => {
            let mut symbols = 0;
            let mut deps = 0;
            if let Ok(new_id) = db.file_by_path(rel).map(|f| f.map(|r| r.id))
                && let Some(id) = new_id
            {
                symbols = db.symbols_for_file(id).map(|s| s.len()).unwrap_or(0);
                deps = db.dependencies_of(id).map(|d| d.len()).unwrap_or(0);
            }
            if before.is_some()
                && before.map(|b| b.hash) == db.file_by_path(rel).ok().flatten().map(|f| f.hash)
            {
                return;
            }
            if t.is_json() {
                emit_watch_json(&serde_json::json!({
                    "event": "changed",
                    "path": rel,
                    "symbols": symbols,
                    "dependencies": deps,
                }));
            } else {
                t.p(&format!(
                    "{} {}  ({} symbols, {} deps)",
                    t.style(Default::CYAN, "changed"),
                    rel,
                    symbols,
                    deps
                ));
            }
        }
        Err(e) => {
            if t.is_json() {
                emit_watch_json(&serde_json::json!({
                    "event": "error",
                    "path": rel,
                    "message": e.to_string(),
                }));
            } else {
                t.e(&format!("failed to index {rel}: {e}"));
            }
        }
    }
}

pub fn watch_loop(project: &Project, t: &Term) -> CtxResult<()> {
    let _ = Path::new("");
    let _ = project;
    let _ = t;
    Ok(())
}
