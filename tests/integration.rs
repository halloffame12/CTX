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
fn relative_imports_with_dot_dot_resolve_to_clean_paths() {
    let root = temp_root("reldots");
    write(
        &root,
        "src/api/users.ts",
        "import { authenticate } from '../auth/auth';\nimport { UserService } from '../users/user';\nexport function handleLogin() { return authenticate('a','b'); }\n",
    );
    write(
        &root,
        "src/auth/auth.ts",
        "import { Session } from './session';\nexport function authenticate(e: string, p: string) { return 1; }\n",
    );
    write(
        &root,
        "src/auth/session.ts",
        "export interface Session { token: string }\n",
    );
    write(&root, "src/users/user.ts", "export class UserService {}\n");
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let users = db.file_by_path("src/api/users.ts").unwrap().unwrap();
    let out = db.internal_dependencies_of(users.id).unwrap();
    assert!(
        out.iter().any(|(p, _)| p == "src/auth/auth.ts"),
        "src/api/users.ts -> src/auth/auth.ts: {out:?}"
    );
    assert!(
        out.iter().any(|(p, _)| p == "src/users/user.ts"),
        "src/api/users.ts -> src/users/user.ts: {out:?}"
    );

    let auth = db.file_by_path("src/auth/auth.ts").unwrap().unwrap();
    let auth_out = db.internal_dependencies_of(auth.id).unwrap();
    assert!(
        auth_out.iter().any(|(p, _)| p == "src/auth/session.ts"),
        "src/auth/auth.ts -> src/auth/session.ts: {auth_out:?}"
    );

    let incoming = db.dependents_of(auth.id).unwrap();
    assert!(
        incoming.iter().any(|(p, _)| p == "src/api/users.ts"),
        "dependents of auth include api/users: {incoming:?}"
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
fn symbol_references_are_symbol_level_not_file_level() {
    // app.py imports only `create_user` from src/models.py — NOT the User
    // class. References for `create_user` must include app.py, while
    // references for `User` must NOT (this was a file-level false positive).
    let root = fixture();
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let create_user = ctx::graph::symbols::symbol_detail(&db, "create_user").unwrap();
    let cu = create_user
        .iter()
        .find(|d| d.file.path == "src/models.py")
        .expect("models.create_user");
    assert!(
        cu.references.iter().any(|(p, _)| p == "src/app.py"),
        "create_user referenced by app.py: {:?}",
        cu.references
    );

    let user = ctx::graph::symbols::symbol_detail(&db, "User").unwrap();
    let u = user
        .iter()
        .find(|d| d.file.path == "src/models.py")
        .expect("models.User");
    assert!(
        !u.references.iter().any(|(p, _)| p == "src/app.py"),
        "User must NOT reference app.py (app only imports create_user): {:?}",
        u.references
    );
}

#[test]
fn ts_symbol_references_are_symbol_level_not_file_level() {
    // The TS grammar has no `import_clause` FIELD on import_statement; the
    // parser must locate the clause by kind or every named import loses its
    // symbol and references degrade to file-level. `app.ts` imports only
    // `createUser` from models.ts — NOT `User` — so User must not be
    // referenced from app.ts.
    let root = temp_root("ts_symref");
    write(
        &root,
        "src/models.ts",
        "export interface User { id: number }\nexport function createUser() {}\n",
    );
    write(
        &root,
        "src/app.ts",
        "import { createUser } from './models';\nconsole.log(createUser());\n",
    );
    write(
        &root,
        "src/other.ts",
        "import { User } from './models';\nconst u: User = { id: 1 };\n",
    );
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let create_user = ctx::graph::symbols::symbol_detail(&db, "createUser").unwrap();
    let cu = create_user
        .iter()
        .find(|d| d.file.path == "src/models.ts")
        .expect("models.createUser");
    assert!(
        cu.references.iter().any(|(p, _)| p == "src/app.ts"),
        "createUser referenced by app.ts: {:?}",
        cu.references
    );

    let user = ctx::graph::symbols::symbol_detail(&db, "User").unwrap();
    let u = user
        .iter()
        .find(|d| d.file.path == "src/models.ts")
        .expect("models.User");
    assert!(
        !u.references.iter().any(|(p, _)| p == "src/app.ts"),
        "User must NOT reference app.ts (app only imports createUser): {:?}",
        u.references
    );
    assert!(
        u.references.iter().any(|(p, _)| p == "src/other.ts"),
        "User referenced by other.ts: {:?}",
        u.references
    );
}

#[test]
fn impact_prefers_production_symbol_over_test_double() {
    let root = temp_root("impact_prod");
    write(&root, "server/index.js", "class SmartMatchmaking {}\n");
    write(
        &root,
        "__tests__/matchmaking.test.js",
        "class SmartMatchmaking {}\n",
    );
    write(
        &root,
        "src/a.js",
        "import { SmartMatchmaking } from '../server/index';\n",
    );
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();
    let (path, _, _sym) = resolve_target(&db, "SmartMatchmaking")
        .unwrap()
        .expect("symbol resolves");
    assert_eq!(
        path, "server/index.js",
        "impact must prefer the production definition over the test mock"
    );
}

#[test]
fn context_puts_direct_hits_first_and_caps_follow_only_files() {
    // A strongly-matching direct file plus a hub with many dependents. The
    // direct hit must be selected first, and the hub's leaves (which have no
    // direct signal) must be capped so they cannot flood the package.
    let root = temp_root("ctx_prio");
    write(
        &root,
        "src/core/api.ts",
        "export function ApiClient() { return 1; }\n",
    );
    for i in 0..12 {
        write(
            &root,
            &format!("src/leaf{i}.ts"),
            // Leaves only import from api.ts — their own symbols must not
            // reference the task keywords, so they are genuinely follow-only
            // (no direct signal) and the follow cap is exercised.
            &format!("import {{ ApiClient }} from '../core/api';\nexport const use{i} = 'x';\n"),
        );
    }
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();
    let pkg =
        ctx::context::builder::build_context(&db, &root, "replace api client", &config, false)
            .unwrap();
    let paths: Vec<&str> = pkg.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(
        paths.first(),
        Some(&"src/core/api.ts"),
        "direct hit must be selected first: {paths:?}"
    );
    let leaves = paths.iter().filter(|p| p.starts_with("src/leaf")).count();
    assert!(
        leaves <= 6,
        "follow-only leaves capped at 6, got {leaves}: {paths:?}"
    );
}

#[test]
fn changed_symbols_reports_only_actual_diffs() {
    let root = temp_root("changed_syms");
    write(
        &root,
        "src/a.ts",
        "export function keep() { return 1; }\nexport function edit() { return 1; }\n",
    );
    let git = ctx::git::GitRepo { root: root.clone() };
    git.run(&["init", "-q"]).unwrap();
    git.run(&["config", "user.email", "qa@example.com"])
        .unwrap();
    git.run(&["config", "user.name", "QA"]).unwrap();
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "initial"]).unwrap();

    // edit only `edit`; `keep` is untouched
    write(
        &root,
        "src/a.ts",
        "export function keep() { return 1; }\nexport function edit(x: number) { return x; }\n",
    );

    let db = ctx::graph::database::Database::open(&root).unwrap();
    let report = ctx::git::changed::changed_symbols(&git, &db, None).unwrap();
    let names: Vec<&str> = report.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"edit"), "edited symbol listed: {names:?}");
    assert!(
        !names.contains(&"keep"),
        "unchanged symbol must not be listed: {names:?}"
    );
    assert!(
        report
            .symbols
            .iter()
            .any(|s| s.name == "edit" && s.status == "Modified"),
        "edit flagged as Modified: {:?}",
        report.symbols
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
fn python_module_scope_and_enum_fields_are_indexed() {
    let root = Path::new(".");
    let py = r#"
DEFAULT_ROLE = "user"

class OrderStatus(Enum):
    PENDING = "pending"
    PAID = "paid"

@dataclass
class Order:
    id: int
    status: OrderStatus = OrderStatus.PENDING

MAX_ITEMS = 100
"#;
    let parsed = parse_source(ctx::lang::LanguageId::Python, py, "order.py", root).unwrap();
    let find = |name: &str, kind: &str| {
        parsed
            .symbols
            .iter()
            .any(|s| s.name == name && s.kind.as_str() == kind)
    };
    // Module-level constants must be indexed (regression: wrapped in expression_statement).
    assert!(
        find("DEFAULT_ROLE", "constant"),
        "module constant DEFAULT_ROLE"
    );
    assert!(find("MAX_ITEMS", "constant"), "module constant MAX_ITEMS");
    // Enum members indexed as fields of the enum class.
    assert!(find("PENDING", "field"), "enum member PENDING");
    assert!(find("PAID", "field"), "enum member PAID");
    // Dataclass fields indexed as fields of the class.
    let order = parsed
        .symbols
        .iter()
        .find(|s| s.name == "Order" && s.kind.as_str() == "class")
        .expect("Order class");
    assert!(
        parsed.symbols.iter().any(|s| {
            s.name == "id"
                && s.kind == ctx::parser::traits::SymbolKind::Field
                && s.parent.as_deref() == Some("Order")
        }),
        "dataclass field id parent=Order"
    );
    assert!(
        parsed.symbols.iter().any(|s| s.name == "status"
            && s.kind == ctx::parser::traits::SymbolKind::Field
            && s.parent.as_deref() == Some("Order")),
        "dataclass field status parent=Order"
    );
    let _ = order;
}

#[test]
fn javascript_field_definition_and_malformed_recovery() {
    let root = Path::new(".");
    let js = r#"
class Cart {
  items = [];

  constructor() {
    this.total = 0;
  }

  addItem(item) {
    this.items.push(item);
  }
}
"#;
    let parsed = parse_source(ctx::lang::LanguageId::JavaScript, js, "cart.js", root).unwrap();
    assert!(
        parsed.symbols.iter().any(|s| {
            s.name == "items"
                && s.kind == ctx::parser::traits::SymbolKind::Field
                && s.parent.as_deref() == Some("Cart")
        }),
        "field_definition uses property field name"
    );

    // Malformed source with unclosed braces: function must still be recovered.
    let broken = "function broken() {\n  if (true) {\n    return 1;";
    let parsed =
        parse_source(ctx::lang::LanguageId::JavaScript, broken, "broken.js", root).unwrap();
    assert!(
        parsed
            .symbols
            .iter()
            .any(|s| s.name == "broken" && s.kind.as_str() == "function"),
        "malformed JS recovers broken()"
    );
}

#[test]
fn go_struct_fields_method_receiver_and_type_alias() {
    let root = Path::new(".");
    let go = r#"package models

type User struct {
	ID   int64
	Name string
}

type OrderID = int64

type UserStatus int

const (
	StatusActive   UserStatus = 1
	StatusInactive UserStatus = 2
)

func (u *User) DisplayName() string {
	return u.Name
}
"#;
    let parsed = parse_source(ctx::lang::LanguageId::Go, go, "user.go", root).unwrap();
    let find = |name: &str, kind: &str| {
        parsed
            .symbols
            .iter()
            .any(|s| s.name == name && s.kind.as_str() == kind)
    };
    // Struct fields nested under field_declaration_list must be indexed.
    assert!(find("ID", "field"), "struct field ID");
    assert!(find("Name", "field"), "struct field Name");
    // type alias `type OrderID = int64` is a type symbol.
    assert!(find("OrderID", "type"), "type alias OrderID");
    // const block: only identifiers registered (no phantom type_identifier consts).
    assert!(find("StatusActive", "constant"), "const StatusActive");
    assert!(find("StatusInactive", "constant"), "const StatusInactive");
    assert!(
        parsed
            .symbols
            .iter()
            .filter(|s| s.name == "UserStatus")
            .count()
            <= 1,
        "UserStatus type registered at most once (no phantom const)"
    );
    // Method on pointer receiver parented to the struct.
    assert!(
        parsed.symbols.iter().any(|s| {
            s.name == "DisplayName"
                && s.kind == ctx::parser::traits::SymbolKind::Method
                && s.parent.as_deref() == Some("User")
        }),
        "receiver method DisplayName parent=User"
    );
}

#[test]
fn rust_struct_fields_generic_impl_and_trait_impl_naming() {
    let root = Path::new(".");
    let rs = r#"
pub struct Order<T> {
    pub id: u64,
    pub items: Vec<T>,
}

impl<T> Order<T> {
    pub fn total(&self) -> u64 {
        self.items.len() as u64
    }
}

pub trait Describe {
    fn describe(&self) -> String;
}

impl Describe for User {
    fn describe(&self) -> String {
        format!("{}", self.id)
    }
}
"#;
    let parsed = parse_source(ctx::lang::LanguageId::Rust, rs, "order.rs", root).unwrap();
    // Struct fields nested under field_declaration_list must be indexed.
    assert!(
        parsed.symbols.iter().any(|s| s.name == "id"
            && s.kind == ctx::parser::traits::SymbolKind::Field
            && s.parent.as_deref() == Some("Order")),
        "struct field id parent=Order"
    );
    assert!(
        parsed.symbols.iter().any(|s| s.name == "items"
            && s.kind == ctx::parser::traits::SymbolKind::Field
            && s.parent.as_deref() == Some("Order")),
        "struct field items parent=Order"
    );
    // Generic impl<T> Order<T> names the impl after the base type.
    assert!(
        parsed
            .symbols
            .iter()
            .any(|s| s.name == "Order" && s.kind == ctx::parser::traits::SymbolKind::Impl),
        "generic impl<T> Order<T> named Order"
    );
    assert!(
        parsed.symbols.iter().any(|s| s.name == "total"
            && s.kind == ctx::parser::traits::SymbolKind::Method
            && s.parent.as_deref() == Some("Order")),
        "total method parent=Order"
    );
    // Trait impl `impl Describe for User` still indexes the trait and methods.
    assert!(
        parsed
            .symbols
            .iter()
            .any(|s| s.name == "Describe" && s.kind.as_str() == "trait"),
        "trait Describe"
    );
    assert!(
        parsed.symbols.iter().any(|s| s.name == "describe"),
        "describe method present"
    );
}

#[test]
fn typescript_field_definition_name_field() {
    let root = Path::new(".");
    let ts = r#"
export class ApiClient {
  private readonly userService: UserService;
  protected version = "2.0";

  constructor(userService: UserService) {
    this.userService = userService;
  }
}
"#;
    let parsed = parse_source(ctx::lang::LanguageId::TypeScript, ts, "client.ts", root).unwrap();
    assert!(
        parsed.symbols.iter().any(|s| {
            s.name == "userService"
                && s.kind == ctx::parser::traits::SymbolKind::Field
                && s.parent.as_deref() == Some("ApiClient")
        }),
        "TS private field userService parent=ApiClient"
    );
    assert!(
        parsed.symbols.iter().any(|s| s.name == "version"
            && s.kind == ctx::parser::traits::SymbolKind::Field
            && s.parent.as_deref() == Some("ApiClient")),
        "TS protected field version parent=ApiClient"
    );
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
fn context_follows_dependencies_of_relevant_files() {
    let root = temp_root("ctx_depfollow");
    // service imports a client; task mentions the service's keyword only.
    write(
        &root,
        "src/service.py",
        "from src.client import Client\nclass Service:\n    def run(self):\n        return Client().go()\n",
    );
    write(
        &root,
        "src/client.py",
        "class Client:\n    def go(self):\n        return 1\n",
    );
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let pkg = ctx::context::build_context(&db, &root, "run the service", &config, false).unwrap();
    let paths: Vec<&str> = pkg.files.iter().map(|f| f.path.as_str()).collect();
    assert!(
        paths.contains(&"src/service.py"),
        "service.py relevant: {paths:?}"
    );
    assert!(
        paths.contains(&"src/client.py"),
        "dependency-following should include client.py: {paths:?}"
    );
    assert!(
        pkg.files.iter().any(
            |f| f.path == "src/client.py" && f.reasons.iter().any(|r| r.contains("dependency"))
        ),
        "client.py should cite the dependency reason"
    );
}

#[test]
fn context_follows_dependents_of_relevant_files() {
    let root = temp_root("ctx_deprev");
    // repository is imported by a service; task names the repository's concept.
    // Changing the repository contract must surface its dependents (the service).
    write(
        &root,
        "src/data/repository.py",
        "class SearchIndex:\n    def query(self):\n        return None\n",
    );
    write(
        &root,
        "src/app/service.py",
        "from src.data.repository import SearchIndex as Store\nclass Worker:\n    def go(self):\n        return Store().load()\n",
    );
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let pkg =
        ctx::context::build_context(&db, &root, "repair the search index", &config, false).unwrap();
    let paths: Vec<&str> = pkg.files.iter().map(|f| f.path.as_str()).collect();
    assert!(
        paths.contains(&"src/data/repository.py"),
        "repository.py relevant: {paths:?}"
    );
    assert!(
        paths.contains(&"src/app/service.py"),
        "dependents-following should include importing service.py: {paths:?}"
    );
    assert!(
        pkg.files
            .iter()
            .any(|f| f.path == "src/app/service.py"
                && f.reasons.iter().any(|r| r.contains("dependent"))),
        "service.py should cite the dependent reason"
    );
}

#[test]
fn context_does_not_flood_dependents_of_hubs() {
    let root = temp_root("ctx_deprev_hub");
    // a shared module imported by many files must not pull all its dependents in.
    write(&root, "src/db.py", "def connect():\n    return 'db'\n");
    for i in 0..60 {
        write(
            &root,
            &format!("src/mod{i}/uses.py"),
            &format!(
                "from src.db import connect\nclass M{i}:\n    def run(self):\n        return connect()\n"
            ),
        );
    }
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let pkg = ctx::context::build_context(&db, &root, "update db module", &config, false).unwrap();
    let deps: Vec<&str> = pkg
        .files
        .iter()
        .map(|f| f.path.as_str())
        .filter(|p| p.contains("uses.py"))
        .collect();
    assert!(
        deps.len() < 10,
        "hub dependents must be capped, got {deps:?}"
    );
}

#[test]
fn context_follows_integration_point_dependents_of_oversized_hub() {
    let root = temp_root("ctx_deprev_bighub");
    // A hub imported by many LEAF modules, plus one genuine integration point
    // (a context provider) that is itself imported by several components.
    // Replacing the hub's contract must surface the context even though the
    // hub has far more than MAX_FOLLOW_DEPENDENTS dependents: the leaf modules
    // are capped, the integration point is followed.
    write(
        &root,
        "web/src/api/client.ts",
        "export class ApiClient {\n  async get<T>(path: string): Promise<T> { return {} as T }\n}\nexport const api = new ApiClient();\n",
    );
    write(
        &root,
        "web/src/context/AuthContext.tsx",
        "import { api } from '../api/client';\nexport function AuthProvider() {\n  return api.get('/me');\n}\n",
    );
    for i in 0..50 {
        write(
            &root,
            &format!("web/src/components/c{i}.tsx"),
            &format!(
                "import {{ AuthProvider }} from '../context/AuthContext';\nexport const C{i} = AuthProvider;\n"
            ),
        );
    }
    for i in 0..60 {
        write(
            &root,
            &format!("web/src/api/module{i}.api.ts"),
            &format!(
                "import {{ api }} from './client';\nexport const module{i}Api = {{\n  fetch() {{ return api.get<{{}}>('/m{i}'); }},\n}};\n"
            ),
        );
    }
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let pkg =
        ctx::context::build_context(&db, &root, "Replace API client", &config, false).unwrap();
    let paths: Vec<&str> = pkg.files.iter().map(|f| f.path.as_str()).collect();
    assert!(
        paths.contains(&"web/src/context/AuthContext.tsx"),
        "integration-point dependent must be followed despite oversized hub: {paths:?}"
    );
    let leaves: Vec<&str> = pkg
        .files
        .iter()
        .map(|f| f.path.as_str())
        .filter(|p| p.contains("module") && p.contains(".api.ts"))
        .collect();
    assert!(
        leaves.len() < 60,
        "leaf dependents of oversized hub must stay capped below the full 60: {leaves:?}"
    );
}

#[test]
fn context_does_not_match_substrings_across_word_boundaries() {
    let root = temp_root("ctx_substr");
    // "warehouseResult1004" contains the letter sequence "user" mid-word
    // ("houser" + "esult") but is NOT about users. A "user" task must not
    // drag it in via naive substring matching.
    write(
        &root,
        "src/db/user.model.ts",
        "export class User {\n  id: number\n}\n",
    );
    write(
        &root,
        "src/warehouse/warehouseResult1004.ts",
        "export interface WarehouseResult1004 {\n  ok: boolean\n  total: number\n}\n",
    );
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let pkg =
        ctx::context::build_context(&db, &root, "change the user model", &config, false).unwrap();
    let paths: Vec<&str> = pkg.files.iter().map(|f| f.path.as_str()).collect();
    assert!(
        paths.contains(&"src/db/user.model.ts"),
        "user.model.ts relevant: {paths:?}"
    );
    assert!(
        !paths.contains(&"src/warehouse/warehouseResult1004.ts"),
        "substring 'user' in warehouseResult1004 must not match, got {paths:?}"
    );
    assert!(
        !pkg.relevant_symbols
            .iter()
            .any(|s| s.name.contains("WarehouseResult")),
        "WarehouseResult1004 symbol must not be relevant: {:?}",
        pkg.relevant_symbols
            .iter()
            .map(|s| &s.name)
            .collect::<Vec<_>>()
    );
}

#[test]
fn context_does_not_flood_hub_with_generic_keyword() {
    let root = temp_root("ctx_idf");
    // A repo full of structurally-identical `*.api.ts` modules that all import
    // a shared client, plus one genuine consumer (a React context) that also
    // imports the client and is itself imported by several components. The
    // task keyword "api" matches every filler module exactly (symbol name
    // "module0Api", path segment "api"); without IDF dampening those fillers
    // flood the package and crowd out the real consumer.
    write(
        &root,
        "web/src/api/client.ts",
        "export class ApiClient {\n  async get<T>(path: string): Promise<T> { return {} as T }\n}\nexport const api = new ApiClient();\n",
    );
    write(
        &root,
        "web/src/context/AuthContext.tsx",
        "import { api } from '../api/client';\nexport function AuthProvider() {\n  return api.get('/me');\n}\n",
    );
    for i in 0..30 {
        write(
            &root,
            &format!("web/src/components/c{i}.tsx"),
            &format!(
                "import {{ AuthProvider }} from '../context/AuthContext';\nexport const C{i} = AuthProvider;\n"
            ),
        );
    }
    for i in 0..30 {
        write(
            &root,
            &format!("web/src/api/module{i}.api.ts"),
            &format!(
                "import {{ api }} from './client';\nexport const module{i}Api = {{\n  fetch() {{ return api.get<{{}}>('/m{i}'); }},\n}};\n"
            ),
        );
    }
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let pkg =
        ctx::context::build_context(&db, &root, "Replace API client", &config, false).unwrap();
    let paths: Vec<&str> = pkg.files.iter().map(|f| f.path.as_str()).collect();
    assert!(
        paths.contains(&"web/src/api/client.ts"),
        "client.ts relevant: {paths:?}"
    );
    assert!(
        paths.contains(&"web/src/context/AuthContext.tsx"),
        "generic keyword 'api' must not crowd out the genuine dependent, got {paths:?}"
    );
}

#[test]
fn context_file_included_via_symbol_match_keeps_reasons() {
    let root = temp_root("ctx_reasons");
    // A file whose only relevance is a symbol match — its path shares no task
    // keyword, no framework/hub/git signals apply. It must still be scored and
    // must surface *why* (the symbol reason) instead of an unexplained empty
    // "reasons" list, so the package stays explainable.
    write(
        &root,
        "src/modules/admin/admin.controller.ts",
        "export async function promoteUser(userId: string): Promise<void> {}\nexport async function demoteUser(userId: string): Promise<void> {}\n",
    );
    write(
        &root,
        "src/modules/admin/admin.routes.ts",
        "import { promoteUser } from './admin.controller';\nexport const routes = [promoteUser];\n",
    );
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let pkg =
        ctx::context::build_context(&db, &root, "Change user schema", &config, false).unwrap();
    let file = pkg
        .files
        .iter()
        .find(|f| f.path == "src/modules/admin/admin.controller.ts");
    assert!(
        file.is_some(),
        "symbol-matched file must be included: {:?}",
        pkg.files
            .iter()
            .map(|f| f.path.as_str())
            .collect::<Vec<_>>()
    );
    let reasons = file.unwrap().reasons.join("; ");
    assert!(
        reasons.contains("promoteUser") || reasons.contains("user"),
        "reasons must explain the symbol match, got: {reasons:?}"
    );
}

#[test]
fn context_uses_synonym_vocabulary() {
    let root = temp_root("ctx_syn");
    write(
        &root,
        "src/auth.py",
        "def authenticate_with_password(user, password):\n    return True\n",
    );
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    // "login" must find `authenticate_with_password` via synonym expansion.
    let pkg = ctx::context::build_context(&db, &root, "login flow", &config, false).unwrap();
    assert!(
        pkg.relevant_symbols
            .iter()
            .any(|s| s.name.contains("authenticate")),
        "login should surface authenticate symbols: {:?}",
        pkg.relevant_symbols
            .iter()
            .map(|s| s.name.clone())
            .collect::<Vec<_>>()
    );
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

#[test]
fn qualified_symbol_lookup_resolves_parent_member() {
    let root = temp_root("qualified");
    write(
        &root,
        "src/user.ts",
        "export interface User { id: string; name: string; }\nexport class UserService {\n  updateUser(id: string) { return id; }\n  updatePhoto(id: string) { return id; }\n}\n",
    );
    write(
        &root,
        "src/app.ts",
        "import { UserService } from './user';\nexport function run() { return UserService; }\n",
    );
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let found = ctx::graph::symbols::resolve_symbol(&db, "UserService.updateUser", None).unwrap();
    assert_eq!(
        found.len(),
        1,
        "qualified name resolves to exactly one symbol: {found:?}"
    );
    assert_eq!(found[0].symbol.name, "updateUser");
    assert_eq!(found[0].symbol.parent.as_deref(), Some("UserService"));

    let (path, _file, symbol) = resolve_target(&db, "UserService.updateUser")
        .unwrap()
        .unwrap();
    assert_eq!(path, "src/user.ts");
    assert_eq!(symbol.as_deref(), Some("updateUser"));

    let bare = ctx::graph::symbols::resolve_symbol(&db, "updateUser", None).unwrap();
    assert_eq!(bare.len(), 1, "bare name still resolves");
}

#[test]
fn rust_crate_resolves_from_nested_crate_root() {
    let root = temp_root("rustcrate");
    write(
        &root,
        "rs/lib.rs",
        "pub mod models;\npub mod api;\n\nuse crate::models::User;\nuse super::api::Client;\n\npub fn handle(u: User) -> User { u }\n",
    );
    write(
        &root,
        "rs/models.rs",
        "pub struct User { pub id: String }\n",
    );
    write(&root, "rs/api.rs", "pub struct Client;\n");
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let lib = db.file_by_path("rs/lib.rs").unwrap().unwrap();
    let deps = db.internal_dependencies_of(lib.id).unwrap();
    let targets: Vec<String> = deps.iter().map(|(p, _)| p.clone()).collect();
    assert!(
        targets.contains(&"rs/models.rs".to_string()),
        "crate::models resolves from nested crate root, got: {targets:?}"
    );
    assert!(
        targets.contains(&"rs/api.rs".to_string()),
        "super::api resolves from nested crate root, got: {targets:?}"
    );

    let (path, _, _) = resolve_target(&db, "User").unwrap().unwrap();
    assert_eq!(path, "rs/models.rs");
}

#[test]
fn changed_reports_deleted_files_as_deleted() {
    let root = temp_root("changed");
    write(&root, "src/a.ts", "export function a() { return 1; }\n");
    write(
        &root,
        "src/b.ts",
        "import { a } from './a';\nexport function b() { return a(); }\n",
    );
    write(&root, "src/c.ts", "export function c() { return 3; }\n");
    let git = ctx::git::GitRepo { root: root.clone() };
    git.run(&["init", "-q"]).unwrap();
    git.run(&["config", "user.email", "qa@example.com"])
        .unwrap();
    git.run(&["config", "user.name", "QA"]).unwrap();
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "initial"]).unwrap();

    std::fs::remove_file(root.join("src/c.ts")).unwrap();
    std::fs::write(root.join("src/b.ts"), "export function b() { return 2; }\n").unwrap();
    write(&root, "src/new.ts", "export function n() { return 99; }\n");

    let files = ctx::git::changed::changed_files(&git, None, true).unwrap();
    let status = |p: &str| {
        files
            .iter()
            .find(|f| f.path == p)
            .map(|f| f.status.as_str())
    };
    assert_eq!(status("src/c.ts"), Some("D"));
    assert_eq!(status("src/b.ts"), Some("M"));
    assert_eq!(status("src/new.ts"), Some("A"));
}

#[test]
fn init_writes_gitignore_for_ctx_dir() {
    let root = temp_root("gitignore");
    write(&root, "src/a.ts", "export function a() { return 1; }\n");
    let git = ctx::git::GitRepo { root: root.clone() };
    git.run(&["init", "-q"]).unwrap();
    git.run(&["config", "user.email", "qa@example.com"])
        .unwrap();
    git.run(&["config", "user.name", "QA"]).unwrap();
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "initial"]).unwrap();

    ctx::commands::init::cmd_init(
        &root,
        false,
        None,
        &ctx::output::Term::new(false, true, true),
    )
    .unwrap();

    let gi = std::fs::read_to_string(root.join(".gitignore")).unwrap_or_default();
    assert!(
        gi.lines().any(|l| l.trim() == ".ctx/"),
        "gitignore has .ctx/: {gi:?}"
    );

    // idempotent: running again does not duplicate
    ctx::commands::init::cmd_init(
        &root,
        false,
        None,
        &ctx::output::Term::new(false, true, true),
    )
    .unwrap();
    let gi2 = std::fs::read_to_string(root.join(".gitignore")).unwrap_or_default();
    assert_eq!(gi2.matches(".ctx/").count(), 1, "no duplicate: {gi2:?}");
}

#[test]
fn stats_reports_index_counts_and_db_size() {
    let root = fixture();
    let config = Config::default();
    let report = run_index(&root, &config).unwrap();

    let project = ctx::commands::Project::open(&root, Some(&root)).unwrap();
    let (files, symbols, deps) = project.db.stats().unwrap();

    let mut out = Vec::new();
    {
        use std::io::Write;
        let mut w = out.by_ref();
        ctx::commands::stats::write_stats(&mut w, &project, false).unwrap();
        let _ = w;
    }
    let text = String::from_utf8(out).unwrap();

    assert_eq!(files as usize, report.total_files - report.skipped, "files");
    assert_eq!(symbols, report.symbols_indexed as i64, "symbols");
    assert_eq!(deps, report.dependencies_indexed as i64, "deps");
    assert!(
        text.contains("files indexed") && text.contains("symbols indexed"),
        "plain output lists files and symbols: {text}"
    );
    assert!(
        text.contains("index.db"),
        "plain output lists db size: {text}"
    );

    let mut out = Vec::new();
    ctx::commands::stats::write_stats(&mut out, &project, true).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["files"], serde_json::json!(files));
    assert_eq!(json["symbols"], serde_json::json!(symbols));
    assert_eq!(json["dependencies"], serde_json::json!(deps));
    assert!(
        json["db_size"].as_u64().unwrap() > 0,
        "db has bytes on disk"
    );
}

#[test]
fn doctor_returns_unhealthy_on_stale_or_uninitialized() {
    // Uninitialized project -> NOT_INITIALIZED exit path (Err Unhealthy).
    let root = temp_root("doctor_uninit");
    write(&root, "src/a.ts", "export function a() { return 1; }\n");
    let t = ctx::output::Term::new(true, true, true);
    let err = ctx::commands::doctor::cmd_doctor(&root, None, &t).unwrap_err();
    assert!(
        matches!(err, ctx::errors::CtxError::Unhealthy(_)),
        "uninitialized doctor must be Unhealthy, got {err:?}"
    );

    // Initialized + fresh -> READY (Ok).
    let config = Config::default();
    run_index(&root, &config).unwrap();
    ctx::commands::doctor::cmd_doctor(&root, None, &t).unwrap();

    // Change file content on disk without re-indexing -> STALE (size change is
    // always detected regardless of mtime granularity).
    write(
        &root,
        "src/a.ts",
        "export function a() { return 1; }\nexport function b() { return 2; }\n",
    );
    let err = ctx::commands::doctor::cmd_doctor(&root, None, &t).unwrap_err();
    assert!(
        matches!(err, ctx::errors::CtxError::Unhealthy(_)),
        "stale doctor must be Unhealthy, got {err:?}"
    );
}

#[test]
fn is_test_file_is_segment_aware() {
    use ctx::graph::impact::is_test_file;
    // True positives: conventional test locations/names.
    for p in [
        "src/__tests__/user.ts",
        "src/test/users.ts",
        "src/tests/models.py",
        "src/user.test.ts",
        "src/user.spec.js",
        "src/test_user.py",
        "src/user_test.go",
        "test_util.ts",
        "test/helpers.ts",
    ] {
        assert!(is_test_file(p), "expected test file: {p}");
    }
    // False positives eliminated: production files that merely contain "test".
    for p in [
        "src/testing.ts",
        "src/contest.ts",
        "src/testing-utils.ts",
        "src/testapp.ts",
        "src/protester.py",
        "src/testable.rs",
    ] {
        assert!(!is_test_file(p), "production file misclassified: {p}");
    }
}

#[test]
fn oversized_files_are_not_upserted() {
    let root = temp_root("oversized");
    write(&root, "small.py", "def small():\n    return 1\n");
    // A supported-language file over the max size gets skipped, not indexed.
    let mut config = Config::default();
    config.index.max_file_size = 40;
    let big = "x = 0\n".repeat(64);
    write(&root, "big.py", &big);
    let report = run_index(&root, &config).unwrap();
    assert_eq!(report.skipped, 1, "big.py skipped");
    let db = ctx::graph::database::Database::open(&root).unwrap();
    assert!(
        db.file_by_path("big.py").unwrap().is_none(),
        "skipped file must not be upserted"
    );
    assert!(db.file_by_path("small.py").unwrap().is_some());
    let (files, _, _) = db.stats().unwrap();
    assert_eq!(files, 1, "only small.py in graph");
}

#[test]
fn symbol_detail_qualified_lookup_lists_methods() {
    let root = temp_root("symdetail");
    write(
        &root,
        "src/user.ts",
        "export class UserService {\n  updateUser(id: string) { return id; }\n  deleteUser(id: string) { return id; }\n}\n",
    );
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    // Qualified lookup must still surface the class's methods.
    let details = ctx::graph::symbols::symbol_detail(&db, "UserService.updateUser").unwrap();
    assert_eq!(details.len(), 1, "qualified symbol resolves");
    assert!(
        details[0]
            .methods
            .iter()
            .any(|m| m.name == "updateUser" || m.name == "deleteUser"),
        "methods listed via qualified lookup: {:?}",
        details[0]
            .methods
            .iter()
            .map(|m| m.name.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn context_budget_counts_included_bodies() {
    let root = temp_root("budget");
    // Big body so the skeleton is small but the full file is large.
    let big_body = "    return 1\n".repeat(2000);
    write(&root, "src/huge.py", &format!("def huge():\n{big_body}"));
    write(&root, "src/util.py", "def helper():\n    return 2\n");
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let skel = ctx::context::build_context(&db, &root, "huge helper", &config, false).unwrap();
    let full = ctx::context::build_context(&db, &root, "huge helper", &config, true).unwrap();

    let skel_tokens = skel.total_tokens;
    let full_tokens = full.total_tokens;
    assert!(
        full_tokens > skel_tokens,
        "include_bodies must charge full-file tokens: {full_tokens} vs {skel_tokens}"
    );
    // The single-file package must not claim a smaller token count than the
    // text actually included.
    let huge = full
        .files
        .iter()
        .find(|f| f.path == "src/huge.py")
        .expect("huge.py included");
    let est = ctx::context::skeleton::estimate_tokens(&huge.skeleton);
    assert!(
        huge.tokens >= est,
        "tokens ({}) under-counts included body ({})",
        huge.tokens,
        est
    );
}

#[test]
fn context_git_changes_considered_is_consultation_flag() {
    let root = temp_root("gitflag");
    write(&root, "src/a.ts", "export function a() { return 1; }\n");
    let git = ctx::git::GitRepo { root: root.clone() };
    git.run(&["init", "-q"]).unwrap();
    git.run(&["config", "user.email", "qa@example.com"])
        .unwrap();
    git.run(&["config", "user.name", "QA"]).unwrap();
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "initial"]).unwrap();
    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    // git exists but nothing changed: the signal was still consulted.
    let pkg =
        ctx::context::build_context_with(&db, &root, "a thing", &config, false, None, Some(&[]))
            .unwrap();
    assert!(
        pkg.git_changes_considered,
        "git signal consulted even with no changes"
    );
}

#[test]
fn single_ref_diff_uses_merge_base_with_head() {
    let root = temp_root("diffmb");
    write(&root, "src/a.ts", "export function a() { return 1; }\n");
    let git = ctx::git::GitRepo { root: root.clone() };
    git.run(&["init", "-q"]).unwrap();
    git.run(&["config", "user.email", "qa@example.com"])
        .unwrap();
    git.run(&["config", "user.name", "QA"]).unwrap();
    git.run(&["symbolic-ref", "HEAD", "refs/heads/main"])
        .unwrap();
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "initial"]).unwrap();
    let initial = git.run(&["rev-parse", "HEAD"]).unwrap().trim().to_string();

    // Main advances past the fork point.
    write(
        &root,
        "src/main_only.ts",
        "export function mainOnly() { return 1; }\n",
    );
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "main advances"]).unwrap();

    // Feature branch forks from the initial commit and adds its own change.
    git.run(&["checkout", "-q", "-b", "feature", &initial])
        .unwrap();
    write(
        &root,
        "src/feature_only.ts",
        "export function featureOnly() { return 2; }\n",
    );
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "feature change"]).unwrap();

    // Single-ref diff (base=main, head=worktree) must resolve base to the
    // merge-base with HEAD: it should report the feature branch's own change,
    // not main's changes since the fork.
    let d = ctx::git::diff::symbol_diff(&git, Some("main"), None, Some(&root)).unwrap();
    eprintln!(
        "DEBUG single-ref base={} files={:?}",
        d.base,
        d.files.iter().map(|f| f.path.clone()).collect::<Vec<_>>()
    );
    let feature = d
        .files
        .iter()
        .find(|f| f.path == "src/feature_only.ts")
        .expect("feature_only.ts in diff");
    assert!(
        feature
            .symbols
            .iter()
            .any(|s| s.status == "Added" && s.name == "featureOnly"),
        "feature branch's own change is Added: {:?}",
        feature
            .symbols
            .iter()
            .map(|s| (&s.status, &s.name))
            .collect::<Vec<_>>()
    );
    assert!(
        d.files.iter().all(|f| f.path != "src/main_only.ts"),
        "main's post-fork change must NOT appear in a single-ref diff: {:?}",
        d.files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>()
    );

    // Two-ref diff (main..feature) still shows both sides directly.
    let d2 = ctx::git::diff::symbol_diff(&git, Some("main"), Some("feature"), Some(&root)).unwrap();
    assert!(
        d2.files.iter().any(|f| f.path == "src/main_only.ts"),
        "two-ref diff includes main's file: {:?}",
        d2.files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>()
    );
}

#[test]
fn diff_respects_explicit_head_ref() {
    let root = temp_root("diffhead");
    write(&root, "src/a.ts", "export function one() { return 1; }\n");
    write(&root, "src/b.ts", "export function two() { return 2; }\n");
    let git = ctx::git::GitRepo { root: root.clone() };
    git.run(&["init", "-q"]).unwrap();
    git.run(&["config", "user.email", "qa@example.com"])
        .unwrap();
    git.run(&["config", "user.name", "QA"]).unwrap();
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "one"]).unwrap();

    // Second commit changes a.ts.
    write(
        &root,
        "src/a.ts",
        "export function one() { return 1; }\nexport function added() { return 3; }\n",
    );
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "two"]).unwrap();

    // Two-ref diff: HEAD~1..HEAD reports the committed change.
    let d0 = ctx::git::diff::symbol_diff(&git, Some("HEAD~1"), Some("HEAD"), Some(&root)).unwrap();
    let a_changed = d0
        .files
        .iter()
        .find(|f| f.path == "src/a.ts")
        .expect("a.ts in diff");
    assert!(
        a_changed
            .symbols
            .iter()
            .any(|s| s.status == "Added" && s.name == "added"),
        "two-ref diff sees committed additions: {:?}",
        a_changed
            .symbols
            .iter()
            .map(|s| (&s.status, &s.name))
            .collect::<Vec<_>>()
    );

    // Uncommitted worktree change must NOT appear in a two-ref diff: b.ts was
    // not touched in HEAD~1..HEAD, so it is not part of the diff at all.
    write(&root, "src/b.ts", "export function two() { return 99; }\n");
    let d1 = ctx::git::diff::symbol_diff(&git, Some("HEAD~1"), Some("HEAD"), Some(&root)).unwrap();
    assert!(
        d1.files.iter().all(|f| f.path != "src/b.ts"),
        "two-ref diff ignores uncommitted worktree changes: {:?}",
        d1.files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>()
    );
    assert_eq!(d1.head, "HEAD", "head label is the explicit ref");
}

#[test]
fn single_ref_diff_excludes_untracked_files() {
    // `ctx diff <ref>` must match `git diff <ref>`: untracked files are
    // visible to `git status` but NOT to `git diff`, so they must not appear
    // in a diff even though `ctx changed` (status semantics) reports them.
    let root = temp_root("diffuntracked");
    write(&root, "src/a.ts", "export function one() { return 1; }\n");
    let git = ctx::git::GitRepo { root: root.clone() };
    git.run(&["init", "-q"]).unwrap();
    git.run(&["config", "user.email", "qa@example.com"])
        .unwrap();
    git.run(&["config", "user.name", "QA"]).unwrap();
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "one"]).unwrap();

    // untracked file, plus a tracked modification
    write(
        &root,
        "src/new.ts",
        "export function fresh() { return 2; }\n",
    );
    write(
        &root,
        "src/a.ts",
        "export function one() { return 1; }\nexport function two() { return 2; }\n",
    );

    let d = ctx::git::diff::symbol_diff(&git, Some("HEAD"), None, Some(&root)).unwrap();
    let paths: Vec<&str> = d.files.iter().map(|f| f.path.as_str()).collect();
    assert!(
        !paths.contains(&"src/new.ts"),
        "untracked file must not appear in diff: {paths:?}"
    );
    assert!(
        paths.contains(&"src/a.ts"),
        "tracked modification appears in diff: {paths:?}"
    );

    // `ctx changed` (git-status semantics) still reports the untracked file.
    let changed = ctx::git::changed::changed_files(&git, None, true).unwrap();
    let cp: Vec<&str> = changed.iter().map(|c| c.path.as_str()).collect();
    assert!(
        cp.contains(&"src/new.ts"),
        "changed reports untracked: {cp:?}"
    );
}

#[test]
fn changed_files_untracked_flag_matches_git_diff_semantics() {
    let root = temp_root("chuntracked");
    write(&root, "src/a.ts", "export function one() { return 1; }\n");
    let git = ctx::git::GitRepo { root: root.clone() };
    git.run(&["init", "-q"]).unwrap();
    git.run(&["config", "user.email", "qa@example.com"])
        .unwrap();
    git.run(&["config", "user.name", "QA"]).unwrap();
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "one"]).unwrap();
    write(
        &root,
        "src/new.ts",
        "export function fresh() { return 2; }\n",
    );

    // status semantics (include_untracked=true) mirrors `git status`.
    let with = ctx::git::changed::changed_files(&git, Some("HEAD"), true).unwrap();
    let wpaths: Vec<&str> = with.iter().map(|c| c.path.as_str()).collect();
    assert!(
        wpaths.contains(&"src/new.ts"),
        "status semantics include untracked: {wpaths:?}"
    );

    // diff semantics (include_untracked=false) mirrors `git diff <ref>`.
    let without = ctx::git::changed::changed_files(&git, Some("HEAD"), false).unwrap();
    let npaths: Vec<&str> = without.iter().map(|c| c.path.as_str()).collect();
    assert!(
        !npaths.contains(&"src/new.ts"),
        "diff semantics exclude untracked: {npaths:?}"
    );
}

#[test]
fn renamed_staged_file_reports_rename_status() {
    // porcelain v1 shows a staged rename as `R  old -> new`; ctx must report
    // the new path with status R, matching `git status`.
    let root = temp_root("rename");
    write(&root, "src/a.ts", "export function one() { return 1; }\n");
    write(&root, "src/b.ts", "export function two() { return 2; }\n");
    let git = ctx::git::GitRepo { root: root.clone() };
    git.run(&["init", "-q"]).unwrap();
    git.run(&["config", "user.email", "qa@example.com"])
        .unwrap();
    git.run(&["config", "user.name", "QA"]).unwrap();
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "one"]).unwrap();
    git.run(&["mv", "src/a.ts", "src/renamed.ts"]).unwrap();

    let files = ctx::git::changed::changed_files(&git, None, true).unwrap();
    let renamed = files
        .iter()
        .find(|f| f.path == "src/renamed.ts")
        .expect("renamed path present");
    assert_eq!(renamed.status, "R", "staged rename status: {files:?}");
    assert_eq!(
        renamed.old_path.as_deref(),
        Some("src/a.ts"),
        "rename keeps source path: {files:?}"
    );
    assert!(
        files.iter().all(|f| f.path != "src/a.ts"),
        "old path must not be reported separately: {files:?}"
    );
}

#[test]
fn pure_rename_reports_no_symbol_changes() {
    // A rename with identical content (git R100) is not a symbol change: the
    // old source must be read from the pre-rename path so nothing is reported
    // as Added, matching `git diff` which shows no content change.
    let root = temp_root("rensymbols");
    write(
        &root,
        "src/a.ts",
        "export function alpha() { return 1; }\nexport function beta() { return 2; }\n",
    );
    let git = ctx::git::GitRepo { root: root.clone() };
    git.run(&["init", "-q"]).unwrap();
    git.run(&["config", "user.email", "qa@example.com"])
        .unwrap();
    git.run(&["config", "user.name", "QA"]).unwrap();
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "one"]).unwrap();
    git.run(&["mv", "src/a.ts", "src/renamed.ts"]).unwrap();

    let d = ctx::git::diff::symbol_diff(&git, Some("HEAD"), None, Some(&root)).unwrap();
    let renamed = d
        .files
        .iter()
        .find(|f| f.path == "src/renamed.ts")
        .expect("renamed file in diff");
    assert_eq!(renamed.status, "R");
    assert!(
        renamed.symbols.is_empty(),
        "pure rename must not report symbol changes: {:?}",
        renamed
            .symbols
            .iter()
            .map(|s| (&s.status, &s.name))
            .collect::<Vec<_>>()
    );
}

#[test]
fn rename_with_edit_reads_old_path_for_symbol_diff() {
    // A committed rename that also edits a function (git reports `R###` with a
    // similarity score) must diff the old symbols against the pre-rename path:
    // the edited function is Modified, the untouched one is not Added.
    let root = temp_root("renedit");
    // Large file so a single-line edit still clears git's ~50% rename
    // threshold (a tiny file comes out as D+A, which git also reports).
    let mut src = String::new();
    for i in 0..200 {
        src.push_str(&format!("export function f{i}() {{ return {i}; }}\n"));
    }
    write(&root, "src/a.ts", &src);
    let git = ctx::git::GitRepo { root: root.clone() };
    git.run(&["init", "-q"]).unwrap();
    git.run(&["config", "user.email", "qa@example.com"])
        .unwrap();
    git.run(&["config", "user.name", "QA"]).unwrap();
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "one"]).unwrap();
    // rename + edit in a single commit: change only the f0 signature.
    git.run(&["mv", "src/a.ts", "src/renamed.ts"]).unwrap();
    let mut edited = String::new();
    edited.push_str("export function f0(x: number): number { return 0; }\n");
    for i in 1..200 {
        edited.push_str(&format!("export function f{i}() {{ return {i}; }}\n"));
    }
    write(&root, "src/renamed.ts", &edited);
    git.run(&["add", "-A"]).unwrap();
    git.run(&["commit", "-qm", "rename+edit"]).unwrap();

    // confirm git reports this as a rename (not D+A) in a two-ref diff
    let raw = git
        .run(&["diff", "--name-status", "HEAD~1", "HEAD"])
        .unwrap();
    assert!(
        raw.lines().any(|l| l.starts_with('R')),
        "git must report a rename for committed rename+edit: {raw:?}"
    );

    let d = ctx::git::diff::symbol_diff(&git, Some("HEAD~1"), Some("HEAD"), Some(&root)).unwrap();
    let renamed = d
        .files
        .iter()
        .find(|f| f.path == "src/renamed.ts")
        .expect("renamed file in diff");
    let symbols: Vec<(&str, &str)> = renamed
        .symbols
        .iter()
        .map(|s| (s.status.as_str(), s.name.as_str()))
        .collect();
    assert!(
        symbols.contains(&("Modified", "f0")),
        "edited symbol reported as Modified: {symbols:?}"
    );
    assert!(
        symbols.len() == 1,
        "only the edited symbol changes: {symbols:?}"
    );
}

#[test]
fn doctor_identifies_corrupt_database_without_crashing() {
    // A garbage .ctx/index.db must produce a full report (status CORRUPT,
    // database.healthy=false, rebuild hint) instead of a bare SQLite error.
    let root = temp_root("doc_corrupt");
    write(&root, "src/a.ts", "export function a() { return 1; }\n");
    let config = Config::default();
    run_index(&root, &config).unwrap();
    std::fs::write(root.join(".ctx/index.db"), "this is not a sqlite database").unwrap();

    let report = ctx::commands::doctor::doctor(&root).unwrap();
    assert_eq!(report.status, "CORRUPT", "status: {:?}", report.status);
    let db = report.database.expect("database section present");
    assert!(!db.healthy, "database flagged unhealthy");
    assert!(
        report.warnings.iter().any(|w| w.contains("--force")),
        "warnings suggest --force: {:?}",
        report.warnings
    );

    // Human path must exit non-zero (Unhealthy) for CORRUPT too.
    let t = ctx::output::Term::new(false, true, true);
    let err = ctx::commands::doctor::cmd_doctor(&root, None, &t).unwrap_err();
    assert!(
        matches!(err, ctx::errors::CtxError::Unhealthy(_)),
        "corrupt doctor must be Unhealthy, got {err:?}"
    );
}

#[test]
fn doctor_identifies_truncated_database_without_crashing() {
    let root = temp_root("doc_trunc");
    write(&root, "src/a.ts", "export function a() { return 1; }\n");
    let config = Config::default();
    run_index(&root, &config).unwrap();

    // Truncate the db mid-file: opening may succeed but reads fail.
    let db_path = root.join(".ctx/index.db");
    let data = std::fs::read(&db_path).unwrap();
    std::fs::write(&db_path, &data[..data.len() / 2]).unwrap();

    let report = ctx::commands::doctor::doctor(&root).unwrap();
    assert_eq!(report.status, "CORRUPT", "status: {:?}", report.status);
    assert!(!report.database.unwrap().healthy);
    assert!(
        report.warnings.iter().any(|w| w.contains("--force")),
        "warnings suggest --force: {:?}",
        report.warnings
    );
}

#[test]
fn doctor_reports_invalid_config_instead_of_crashing() {
    let root = temp_root("doc_badcfg");
    write(&root, "src/a.ts", "export function a() { return 1; }\n");
    let config = Config::default();
    run_index(&root, &config).unwrap();
    write(&root, ".ctx/config.toml", "[index\nexclude = [");

    let report = ctx::commands::doctor::doctor(&root).unwrap();
    assert_eq!(report.status, "CONFIG", "status: {:?}", report.status);
    assert!(
        report.warnings.iter().any(|w| w.contains("config.toml")),
        "warnings mention config: {:?}",
        report.warnings
    );
    // The index itself is still healthy; the problem is the config.
    assert!(report.database.unwrap().healthy);

    let t = ctx::output::Term::new(false, true, true);
    let err = ctx::commands::doctor::cmd_doctor(&root, None, &t).unwrap_err();
    assert!(matches!(err, ctx::errors::CtxError::Unhealthy(_)));
}

#[test]
fn doctor_warns_when_config_is_missing() {
    let root = temp_root("doc_missingcfg");
    write(&root, "src/a.ts", "export function a() { return 1; }\n");
    let config = Config::default();
    run_index(&root, &config).unwrap();
    // run_index does not write config.toml; simulate a config that existed
    // and was removed by writing one first, then deleting it.
    write(&root, ".ctx/config.toml", "[index]\n");
    std::fs::remove_file(root.join(".ctx/config.toml")).unwrap();

    let report = ctx::commands::doctor::doctor(&root).unwrap();
    assert_eq!(
        report.status, "READY",
        "index still healthy: {:?}",
        report.status
    );
    assert!(
        report.warnings.iter().any(|w| w.contains("missing")),
        "warnings mention missing config: {:?}",
        report.warnings
    );
}

#[test]
fn doctor_warns_when_project_directory_missing() {
    let root = temp_root("doc_missing_dir");
    // Never create the directory.
    let report = ctx::commands::doctor::doctor(&root).unwrap();
    assert_eq!(report.status, "NOT_INITIALIZED");
    assert!(
        report.warnings.iter().any(|w| w.contains("does not exist")),
        "warnings mention missing dir: {:?}",
        report.warnings
    );
}

#[test]
fn init_hints_force_on_corrupt_index() {
    let root = temp_root("init_corrupt");
    write(&root, "src/a.ts", "export function a() { return 1; }\n");
    let config = Config::default();
    run_index(&root, &config).unwrap();
    std::fs::write(root.join(".ctx/index.db"), "garbage bytes").unwrap();

    let t = ctx::output::Term::new(false, true, true);
    let err = ctx::commands::init::cmd_init(&root, false, None, &t).unwrap_err();
    assert!(
        err.to_string().contains("--force"),
        "non-force init on corrupt index suggests --force, got: {err}"
    );
    assert!(
        err.to_string().contains("corrupt"),
        "error mentions corruption, got: {err}"
    );

    // --force must recover.
    ctx::commands::init::cmd_init(&root, true, None, &t).unwrap();
    assert!(ctx::graph::database::Database::exists(&root));
}

#[test]
fn init_errors_clearly_when_ctx_is_a_file() {
    let root = temp_root("init_ctxfile");
    write(&root, "src/a.ts", "export function a() { return 1; }\n");
    write(&root, ".ctx", "this is a file, not a directory");

    let t = ctx::output::Term::new(false, true, true);
    let err = ctx::commands::init::cmd_init(&root, false, None, &t).unwrap_err();
    assert!(
        err.to_string().contains("not a directory"),
        "clear error when .ctx is a file, got: {err}"
    );
}

// ---- dependency graph regression tests ----

/// Helper: index a project and return the set of internal edges as
/// `(source, target)` project-relative paths.
fn internal_edges(root: &std::path::Path) -> std::collections::BTreeSet<(String, String)> {
    let config = Config::default();
    run_index(root, &config).unwrap();
    let db = ctx::graph::database::Database::open(root).unwrap();
    let mut edges = std::collections::BTreeSet::new();
    let all = db.all_files().unwrap();
    let path_of: std::collections::HashMap<i64, String> =
        all.iter().map(|f| (f.id, f.path.clone())).collect();
    for dep in db
        .conn()
        .prepare("SELECT source_file_id, target_file_id FROM dependencies WHERE target_file_id IS NOT NULL")
        .unwrap()
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
    {
        if let (Some(s), Some(t)) = (path_of.get(&dep.0), path_of.get(&dep.1)) {
            edges.insert((s.clone(), t.clone()));
        }
    }
    edges
}

#[test]
fn python_relative_and_package_imports_resolve_internally() {
    let root = temp_root("dep_py_rel");
    write(&root, "src/pkg/__init__.py", "");
    write(&root, "src/pkg/db.py", "def connect():\n    return 'db'\n");
    write(&root, "src/pkg/models/__init__.py", "");
    write(
        &root,
        "src/pkg/models/user.py",
        "from .. import db\n\ndef get_user():\n    return db.connect()\n",
    );
    write(&root, "src/pkg/api/__init__.py", "");
    write(
        &root,
        "src/pkg/api/client.py",
        "from ..models.user import get_user\n\ndef client():\n    return get_user()\n",
    );
    write(
        &root,
        "src/pkg/top.py",
        "from src.pkg.models import user\n\ndef top():\n    return user.get_user()\n",
    );

    let edges = internal_edges(&root);
    assert!(
        edges.contains(&("src/pkg/models/user.py".into(), "src/pkg/db.py".into())),
        "from .. import db -> db.py: {edges:?}"
    );
    assert!(
        edges.contains(&(
            "src/pkg/api/client.py".into(),
            "src/pkg/models/user.py".into()
        )),
        "from ..models.user import get_user -> user.py: {edges:?}"
    );
    assert!(
        edges.contains(&("src/pkg/top.py".into(), "src/pkg/models/__init__.py".into())),
        "from src.pkg.models import user -> package __init__.py: {edges:?}"
    );
}

#[test]
fn typescript_dynamic_import_and_require_resolve_internally() {
    let root = temp_root("dep_ts_dynamic");
    write(
        &root,
        "src/dyn.ts",
        "export async function dyn() {\n  const m = await import(\"./lazy\");\n  return m.lazy();\n}\n",
    );
    write(
        &root,
        "src/lazy.ts",
        "export function lazy() { return 'lazy'; }\n",
    );
    write(
        &root,
        "src/req.ts",
        "const m = require(\"./cjs\");\nexport const v = m.v;\n",
    );
    write(&root, "src/cjs.js", "module.exports = { v: 1 };\n");

    let edges = internal_edges(&root);
    assert!(
        edges.contains(&("src/dyn.ts".into(), "src/lazy.ts".into())),
        "dynamic import(\"./lazy\"): {edges:?}"
    );
    assert!(
        edges.contains(&("src/req.ts".into(), "src/cjs.js".into())),
        "require(\"./cjs\"): {edges:?}"
    );
}

#[test]
fn typescript_alias_import_resolves_via_src() {
    let root = temp_root("dep_ts_alias");
    write(
        &root,
        "src/alias.ts",
        "import { target } from \"@/alias/target\";\nexport function alias() { return target(); }\n",
    );
    write(
        &root,
        "src/alias/target.ts",
        "export function target() { return 't'; }\n",
    );

    let edges = internal_edges(&root);
    assert!(
        edges.contains(&("src/alias.ts".into(), "src/alias/target.ts".into())),
        "@/alias/target should resolve via src/: {edges:?}"
    );
}

#[test]
fn typescript_workspace_package_resolves_cross_package() {
    let root = temp_root("dep_ts_workspace");
    write(
        &root,
        "package.json",
        r#"{ "name": "root", "workspaces": ["packages/*"] }"#,
    );
    write(
        &root,
        "packages/core/package.json",
        r#"{ "name": "@acme/core", "main": "src/index.ts" }"#,
    );
    write(
        &root,
        "packages/core/src/index.ts",
        "export function coreFn() { return 'core'; }\n",
    );
    write(
        &root,
        "packages/app/package.json",
        r#"{ "name": "@acme/app", "dependencies": { "@acme/core": "*" } }"#,
    );
    write(
        &root,
        "packages/app/src/main.ts",
        "import { coreFn } from \"@acme/core\";\nexport function app() { return coreFn(); }\n",
    );

    let edges = internal_edges(&root);
    assert!(
        edges.contains(&(
            "packages/app/src/main.ts".into(),
            "packages/core/src/index.ts".into()
        )),
        "workspace cross-package dep: {edges:?}"
    );
}

#[test]
fn typescript_barrel_index_and_reexport_resolve() {
    let root = temp_root("dep_ts_barrel");
    write(&root, "src/barrel/one.ts", "export const one = 1;\n");
    write(&root, "src/barrel/two.ts", "export const two = 2;\n");
    write(
        &root,
        "src/barrel/index.ts",
        "export * from \"./one\";\nexport { two } from \"./two\";\n",
    );
    write(
        &root,
        "src/barrel-consumer.ts",
        "import { one, two } from \"./barrel\";\nexport const sum = one + two;\n",
    );
    write(
        &root,
        "src/reexport/main.ts",
        "export { x } from \"./mod\";\n",
    );
    write(&root, "src/reexport/mod.ts", "export const x = 1;\n");

    let edges = internal_edges(&root);
    assert!(
        edges.contains(&("src/barrel/index.ts".into(), "src/barrel/one.ts".into())),
        "barrel index -> one: {edges:?}"
    );
    assert!(
        edges.contains(&("src/barrel/index.ts".into(), "src/barrel/two.ts".into())),
        "barrel index -> two: {edges:?}"
    );
    assert!(
        edges.contains(&(
            "src/barrel-consumer.ts".into(),
            "src/barrel/index.ts".into()
        )),
        "consumer -> barrel index: {edges:?}"
    );
    assert!(
        edges.contains(&("src/reexport/main.ts".into(), "src/reexport/mod.ts".into())),
        "re-export -> mod: {edges:?}"
    );
}

#[test]
fn go_module_relative_import_resolves_internally() {
    let root = temp_root("dep_go_mod");
    write(&root, "go.mod", "module example.com/ctxdeps\n\ngo 1.21\n");
    write(
        &root,
        "main.go",
        "package main\n\nimport (\n\t\"example.com/ctxdeps/models\"\n)\n\nfunc main() {\n\tmodels.Run()\n}\n",
    );
    write(
        &root,
        "models/models.go",
        "package models\n\nimport \"example.com/ctxdeps/store\"\n\nfunc Run() {\n\tstore.Store()\n}\n",
    );
    write(
        &root,
        "store/store.go",
        "package store\n\nfunc Store() {}\n",
    );

    let edges = internal_edges(&root);
    assert!(
        edges.contains(&("main.go".into(), "models/models.go".into())),
        "main -> models: {edges:?}"
    );
    assert!(
        edges.contains(&("models/models.go".into(), "store/store.go".into())),
        "models -> store: {edges:?}"
    );
}

#[test]
fn rust_use_source_raw_is_clean_path_not_statement() {
    let root = Path::new(".");
    let src = "use serde::Serialize;\nuse std::collections::HashMap;\n";
    let parsed = parse_source(ctx::lang::LanguageId::Rust, src, "ext.rs", root).unwrap();
    let serde_dep = parsed
        .dependencies
        .iter()
        .find(|d| d.source_raw == "serde::Serialize")
        .expect("serde dep with clean source_raw");
    assert!(matches!(
        serde_dep.resolved,
        ResolvedDependency::External(_)
    ));
    let std_dep = parsed
        .dependencies
        .iter()
        .find(|d| d.source_raw == "std::collections::HashMap")
        .expect("std dep with clean source_raw");
    assert!(matches!(std_dep.resolved, ResolvedDependency::External(_)));
}

#[test]
fn python_import_os_classifies_external_not_unresolved() {
    let root = Path::new(".");
    let src = "import os\nimport requests\nimport totally_missing_module\n";
    let parsed = parse_source(ctx::lang::LanguageId::Python, src, "ext.py", root).unwrap();
    for dep in &parsed.dependencies {
        assert!(
            matches!(dep.resolved, ResolvedDependency::External(_)),
            "python bare module {dep:?} should be External"
        );
    }
}

#[test]
fn python_relative_import_of_missing_module_is_unresolved() {
    let root = Path::new(".");
    let src = "from . import missing_module\n";
    let parsed = parse_source(ctx::lang::LanguageId::Python, src, "x.py", root).unwrap();
    assert!(
        parsed
            .dependencies
            .iter()
            .any(|d| matches!(d.resolved, ResolvedDependency::Unresolved(_))),
        "relative import of missing module is Unresolved, got {:?}",
        parsed.dependencies
    );
}

#[test]
fn impact_analysis_walks_indirect_dependents_and_is_cycle_safe() {
    let root = temp_root("dep_impact");
    write(
        &root,
        "a.ts",
        "import { b } from './b';\nexport function a() { return b(); }\n",
    );
    write(
        &root,
        "b.ts",
        "import { c } from './c';\nexport function b() { return c(); }\n",
    );
    write(
        &root,
        "c.ts",
        "import { d } from './d';\nexport function c() { return d(); }\n",
    );
    write(&root, "d.ts", "export function d() { return 'd'; }\n");

    // cycle: e -> f -> g -> e
    write(
        &root,
        "e.ts",
        "import { f } from './f';\nexport function e() { return f(); }\n",
    );
    write(
        &root,
        "f.ts",
        "import { g } from './g';\nexport function f() { return g(); }\n",
    );
    write(
        &root,
        "g.ts",
        "import { e } from './e';\nexport function g() { return e(); }\n",
    );

    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();

    let (path, id, _) = ctx::graph::impact::resolve_target(&db, "d.ts")
        .unwrap()
        .expect("d.ts exists");
    assert_eq!(path, "d.ts");
    let report = ctx::graph::impact::impact(&db, &path, id, None, 5).unwrap();
    let direct: Vec<&str> = report.direct.iter().map(|f| f.path.as_str()).collect();
    let indirect: Vec<&str> = report.indirect.iter().map(|f| f.path.as_str()).collect();
    assert!(
        direct.contains(&"c.ts"),
        "c.ts is a direct dependent: {direct:?}"
    );
    assert!(indirect.contains(&"b.ts"), "b.ts indirect: {indirect:?}");
    assert!(indirect.contains(&"a.ts"), "a.ts indirect: {indirect:?}");

    // Cycle must not loop forever; BFS visits each file once.
    let (path, id, _) = ctx::graph::impact::resolve_target(&db, "e.ts")
        .unwrap()
        .expect("e.ts exists");
    let report = ctx::graph::impact::impact(&db, &path, id, None, 10).unwrap();
    let all: Vec<&str> = report
        .direct
        .iter()
        .chain(report.indirect.iter())
        .map(|f| f.path.as_str())
        .collect();
    assert_eq!(all.len(), 2, "cycle reachable set is finite: {all:?}");
    assert!(all.contains(&"f.ts") && all.contains(&"g.ts"));
}

#[test]
fn duplicate_imports_do_not_duplicate_edges() {
    let root = temp_root("dep_dupes");
    write(
        &root,
        "a.ts",
        "import { b } from './b';\nimport { c } from './c';\n",
    );
    write(&root, "b.ts", "export function b() { return 1; }\n");
    write(&root, "c.ts", "export function c() { return 2; }\n");

    let config = Config::default();
    run_index(&root, &config).unwrap();
    let db = ctx::graph::database::Database::open(&root).unwrap();
    let a = db.file_by_path("a.ts").unwrap().unwrap();
    let deps = db.dependencies_of(a.id).unwrap();
    assert_eq!(deps.len(), 2, "exactly two import edges: {deps:?}");
}
