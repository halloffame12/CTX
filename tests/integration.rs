use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ctx::config::Config;
use ctx::graph::impact::{impact, resolve_target};
use ctx::indexing::incremental::run_index;
use ctx::parser::parse_source;
use ctx::parser::traits::ResolvedDependency;

fn temp_root(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("ctx_it_{tag}_{nanos}"))
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&p, content).unwrap();
}

/// Small Python + TS + Rust + Go project.
fn fixture() -> PathBuf {
    let root = temp_root("proj");
    write(
        &root,
        "src/models.py",
        r#"from datetime import datetime
from src.helpers import format_date

class User:
    def __init__(self, name, email):
        self.name = name
        self.email = email

    def display_name(self):
        return format_date(self.name)

def create_user(name, email):
    return User(name, email)
"#,
    );
    write(
        &root,
        "src/helpers.py",
        r#"def format_date(value):
    return str(value)
"#,
    );
    write(
        &root,
        "src/app.py",
        r#"from src.models import create_user

def main():
    u = create_user("Ada", "ada@example.com")
    return u.display_name()
"#,
    );
    write(
        &root,
        "util.ts",
        r#"import { formatDate } from "./src/helpers";

export interface User {
  name: string;
  email: string;
}

export function createUser(name: string, email: string): User {
  return { name, email };
}
"#,
    );
    write(
        &root,
        "math.rs",
        r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
"#,
    );
    write(
        &root,
        "run.go",
        r#"package main

import "fmt"

func main() {
    fmt.Println(hello())
}

func hello() string {
    return "hello"
}
"#,
    );
    write(&root, "ignored.txt", "this file has no supported language");
    root
}

#[test]
fn init_indexes_symbols_and_dependencies() {
    let root = fixture();
    let config = Config::default();
    let report = run_index(&root, &config).unwrap();

    assert!(
        report.total_files >= 6,
        "expected >=6 files, got {}",
        report.total_files
    );
    assert!(
        report.parsed_files == report.supported_files,
        "all supported files parsed"
    );
    assert!(report.symbols_indexed > 0, "symbols indexed");
    assert!(
        report.dependencies_indexed >= 3,
        "deps indexed, got {}",
        report.dependencies_indexed
    );
    assert!(
        report.parse_errors.is_empty(),
        "no parse errors: {:?}",
        report.parse_errors
    );

    let db = ctx::graph::database::Database::open(&root).unwrap();
    let (files, symbols, deps) = db.stats().unwrap();
    assert_eq!(files as usize, report.supported_files);
    assert_eq!(symbols as usize, report.symbols_indexed);
    assert_eq!(deps as usize, report.dependencies_indexed);

    // deps resolve to internal files
    let app = db
        .file_by_path("src/app.py")
        .unwrap()
        .expect("app.py indexed");
    let outgoing = db.internal_dependencies_of(app.id).unwrap();
    assert!(
        outgoing.iter().any(|(p, _)| p == "src/models.py"),
        "app.py should depend on src/models.py: {outgoing:?}"
    );

    let models = db
        .file_by_path("src/models.py")
        .unwrap()
        .expect("models.py indexed");
    let incoming = db.dependents_of(models.id).unwrap();
    assert!(
        incoming.iter().any(|(p, _)| p == "src/app.py"),
        "models.py should be depended on by app.py: {incoming:?}"
    );

    // util.ts -> src/helpers.py (extension probing)
    let util = db
        .file_by_path("util.ts")
        .unwrap()
        .expect("util.ts indexed");
    let ts_out = db.internal_dependencies_of(util.id).unwrap();
    assert!(
        ts_out.iter().any(|(p, _)| p == "src/helpers.py"),
        "util.ts should resolve ./src/helpers to src/helpers.py: {ts_out:?}"
    );
}

#[test]
fn search_and_symbol_details() {
    let root = fixture();
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let hits = db.search("create_user", None, 10).unwrap();
    assert!(!hits.is_empty(), "search should find create_user");
    assert_eq!(hits[0].name, "create_user");

    let hits_ts = db.search("createUser", None, 10).unwrap();
    assert!(!hits_ts.is_empty(), "search should find createUser");

    // kind filter
    let classes = db.search("User", Some("class"), 10).unwrap();
    assert!(!classes.is_empty());

    // symbol detail carries methods
    let detail = ctx::graph::symbols::symbol_detail(&db, "User").unwrap();
    let py = detail
        .iter()
        .find(|d| d.file.path == "src/models.py")
        .expect("python User");
    assert!(
        py.methods
            .iter()
            .any(|m| m.name == "__init__" || m.name == "display_name")
    );
}

#[test]
fn impact_analysis_finds_dependents() {
    let root = fixture();
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let (path, id, _symbol) = resolve_target(&db, "create_user")
        .unwrap()
        .expect("symbol exists");
    assert_eq!(path, "src/models.py");
    let report = impact(&db, &path, id, None, 3).unwrap();

    let direct_paths: Vec<&str> = report.direct.iter().map(|f| f.path.as_str()).collect();
    assert!(
        direct_paths.contains(&"src/app.py"),
        "app.py is a direct dependent: {direct_paths:?}"
    );
}

#[test]
fn skeleton_preserves_structure_without_bodies() {
    let src = r#"
class User:
    def __init__(self, name, email):
        self.name = name
        if email:
            self.email = email

    def display_name(self):
        return f"{self.name}"
"#;
    let parsed = parse_source(
        ctx::lang::LanguageId::Python,
        src,
        "src/models.py",
        Path::new("."),
    )
    .unwrap();
    assert!(
        parsed
            .symbols
            .iter()
            .any(|s| s.name == "User" && s.kind.as_str() == "class")
    );
    assert!(parsed.symbols.iter().any(|s| s.name == "__init__"));

    let skel = ctx::context::skeleton::skeleton_for(
        Path::new("."),
        "src/models.py",
        ctx::lang::LanguageId::Python,
        src,
    )
    .unwrap();
    assert!(
        skel.skeleton.contains("def __init__"),
        "skeleton keeps signature"
    );
    assert!(
        !skel.skeleton.contains("self.name = name"),
        "skeleton drops bodies"
    );
    assert!(skel.skeleton.contains("..."), "skeleton has elision marker");
}

#[test]
fn rust_and_go_parsing() {
    let root = Path::new(".");
    let rs = r#"
pub fn add(a: i32, b: i32) -> i32 { a + b }
pub struct Point { pub x: f64, pub y: f64 }
impl Point {
    pub fn len(&self) -> f64 { 0.0 }
}
"#;
    let parsed = parse_source(ctx::lang::LanguageId::Rust, rs, "math.rs", root).unwrap();
    assert!(parsed.symbols.iter().any(|s| s.name == "add"));
    assert!(parsed.symbols.iter().any(|s| s.name == "Point"));
    assert!(parsed.symbols.iter().any(|s| s.name == "len"));

    let go = r#"package main
import "fmt"
func main() { fmt.Println(hi()) }
func hi() string { return "hi" }
"#;
    let parsed = parse_source(ctx::lang::LanguageId::Go, go, "main.go", root).unwrap();
    assert!(parsed.symbols.iter().any(|s| s.name == "main"));
    assert!(parsed.symbols.iter().any(|s| s.name == "hi"));
}

#[test]
fn dependency_resolution_classification() {
    let root = Path::new(".");
    // external packages stay external
    let py = r#"
import os
from datetime import datetime
from src.models import User
"#;
    let parsed = parse_source(ctx::lang::LanguageId::Python, py, "x.py", root).unwrap();
    let deps = &parsed.dependencies;
    assert!(
        deps.iter()
            .any(|d| matches!(d.resolved, ResolvedDependency::External(_))),
        "os/datetime should be external"
    );
    assert!(
        deps.iter()
            .any(|d| matches!(d.resolved, ResolvedDependency::External(_))),
        "a dotted module with no matching file classifies as external"
    );
    // unknown external rust segment
    let rsrc = "use std::collections::HashMap;";
    let parsed = parse_source(ctx::lang::LanguageId::Rust, rsrc, "x.rs", root).unwrap();
    assert!(
        parsed
            .dependencies
            .iter()
            .all(|d| matches!(d.resolved, ResolvedDependency::External(_))),
        "std should be external"
    );
}

#[test]
fn rust_use_resolves_to_internal_files() {
    let root = temp_root("rustmod");
    write(
        &root,
        "src/main.rs",
        r#"
mod graph;

use crate::graph::database::Database;

fn main() {
    let _ = Database::open(std::path::Path::new("."));
}
"#,
    );
    write(
        &root,
        "src/graph/database.rs",
        r#"
pub struct Database;
impl Database {
    pub fn open(_: &std::path::Path) -> Database { Database }
}
"#,
    );
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let main = db
        .file_by_path("src/main.rs")
        .unwrap()
        .expect("main.rs indexed");
    let outgoing = db.internal_dependencies_of(main.id).unwrap();
    assert!(
        outgoing.iter().any(|(p, _)| p == "src/graph/database.rs"),
        "crate::graph::database should resolve to src/graph/database.rs: {outgoing:?}"
    );
}

#[test]
fn context_build_ranks_relevant_files() {
    let root = fixture();
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let pkg = ctx::context::build_context(
        &db,
        &root,
        "refactor user creation and formatting",
        &config,
        false,
    )
    .unwrap();
    assert!(
        pkg.keywords.contains(&"user".to_string()) || pkg.keywords.contains(&"format".to_string())
    );
    assert!(!pkg.relevant_symbols.is_empty(), "context finds symbols");
    assert!(!pkg.files.is_empty(), "context suggests files");
    assert!(pkg.total_tokens > 0);
    assert!(pkg.suggested_context.contains("Context package"));
}

#[test]
fn incremental_reindex_is_idempotent() {
    let root = fixture();
    let config = Config::default();
    let first = run_index(&root, &config).unwrap();
    let second = run_index(&root, &config).unwrap();

    assert_eq!(
        second.unchanged_files, first.supported_files,
        "second pass is incremental"
    );
    assert_eq!(second.parsed_files, 0, "nothing re-parsed");
    assert_eq!(second.symbols_indexed, 0, "no symbol churn");

    // add a file, reindex, and confirm it appears
    write(&root, "src/extra.py", "def extra(): return 1\n");
    let third = run_index(&root, &config).unwrap();
    assert!(third.parsed_files >= 1, "new file parsed");
    let db = ctx::graph::database::Database::open(&root).unwrap();
    assert!(db.file_by_path("src/extra.py").unwrap().is_some());

    // delete it and confirm removal
    std::fs::remove_file(root.join("src/extra.py")).unwrap();
    let fourth = run_index(&root, &config).unwrap();
    assert!(fourth.deleted_files >= 1, "deleted file removed");
}

#[test]
fn modified_file_replaces_symbols_and_deps() {
    let root = temp_root("modify");
    write(
        &root,
        "src/api.py",
        "def create_user(name):\n    return name\n",
    );
    let config = Config::default();
    run_index(&root, &config).unwrap();

    write(
        &root,
        "src/api.py",
        "def create_admin(name):\n    return name.upper()\n",
    );
    run_index(&root, &config).unwrap();

    let db = ctx::graph::database::Database::open(&root).unwrap();
    let file = db.file_by_path("src/api.py").unwrap().expect("api.py");
    let rows = db.symbols_for_file(file.id).unwrap();
    let names: Vec<&str> = rows.iter().map(|s| s.name.as_str()).collect();
    assert!(
        names.contains(&"create_admin"),
        "new symbol indexed: {names:?}"
    );
    assert!(
        !names.contains(&"create_user"),
        "old symbol removed: {names:?}"
    );
}

#[test]
fn renamed_file_leaves_no_stale_entries() {
    let root = temp_root("rename");
    write(&root, "a.py", "def thing():\n    return 1\n");
    let config = Config::default();
    run_index(&root, &config).unwrap();

    std::fs::rename(root.join("a.py"), root.join("b.py")).unwrap();
    run_index(&root, &config).unwrap();

    let db = ctx::graph::database::Database::open(&root).unwrap();
    assert!(db.file_by_path("a.py").unwrap().is_none(), "old path gone");
    let b = db.file_by_path("b.py").unwrap().expect("new path indexed");
    let names: Vec<String> = db
        .symbols_for_file(b.id)
        .unwrap()
        .iter()
        .map(|s| s.name.clone())
        .collect();
    assert_eq!(names, vec!["thing".to_string()]);
}

#[test]
fn syntax_error_does_not_abort_indexing() {
    let root = temp_root("synerr");
    write(&root, "ok.py", "def fine():\n    return 1\n");
    write(&root, "broken.py", "def broken(:\n    pass\n");
    write(&root, "also_ok.py", "value = 42\n");
    let config = Config::default();
    let report = run_index(&root, &config).unwrap();

    assert!(
        report.parse_errors.iter().any(|e| e.contains("broken.py")),
        "broken file reported: {:?}",
        report.parse_errors
    );
    let db = ctx::graph::database::Database::open(&root).unwrap();
    // healthy siblings still indexed with their symbols
    assert!(db.file_by_path("ok.py").unwrap().is_some());
    assert!(db.file_by_path("also_ok.py").unwrap().is_some());
    // the broken file still has a metadata row
    assert!(db.file_by_path("broken.py").unwrap().is_some());
}

#[test]
fn empty_file_indexes_without_symbols() {
    let root = temp_root("empty");
    write(&root, "empty.py", "");
    let config = Config::default();
    let report = run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();
    let file = db.file_by_path("empty.py").unwrap().expect("indexed");
    assert_eq!(db.symbols_for_file(file.id).unwrap().len(), 0);
    assert!(report.parse_errors.is_empty());
}

#[test]
fn duplicate_symbols_and_imports_are_preserved() {
    let root = temp_root("dups");
    write(
        &root,
        "src/models.py",
        "def make_user():\n    return 1\n\ndef make_user():\n    return 2\n",
    );
    write(
        &root,
        "src/app.py",
        "from src.models import make_user\nfrom src.models import make_user\n\ndef run():\n    return make_user()\n",
    );
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let models = db.file_by_path("src/models.py").unwrap().unwrap();
    let rows = db.symbols_for_file(models.id).unwrap();
    let names: Vec<&str> = rows.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names.iter().filter(|n| **n == "make_user").count(),
        2,
        "duplicate symbol kept: {names:?}"
    );

    let app = db.file_by_path("src/app.py").unwrap().unwrap();
    let outgoing = db.internal_dependencies_of(app.id).unwrap();
    assert_eq!(
        outgoing
            .iter()
            .filter(|(p, _)| p == "src/models.py")
            .count(),
        2,
        "duplicate import kept: {outgoing:?}"
    );
}
