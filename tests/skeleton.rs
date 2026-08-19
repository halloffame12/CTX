//! Skeletonization golden tests: structural info preserved, implementation
//! detail removed, output is deterministic and balanced for brace languages.

use std::path::Path;

use ctx::context::skeleton::skeleton_for;
use ctx::lang::LanguageId;
use ctx::parser::skeletonize;

fn skel(lang: LanguageId, src: &str) -> String {
    skeleton_for(Path::new("."), "x", lang, src)
        .unwrap()
        .skeleton
}

/// Quick structural sanity check: braces/parens balance and no doubled-up
/// placeholder that would indicate overlapping replacements.
fn assert_balanced(skel: &str) {
    for pair in [('{', '}'), ('(', ')'), ('[', ']')] {
        let opens = skel.matches(pair.0).count();
        let closes = skel.matches(pair.1).count();
        assert_eq!(
            opens, closes,
            "unbalanced `{}`/`{}` in skeleton:\n{skel}",
            pair.0, pair.1
        );
    }
}

#[test]
fn deterministic_output() {
    let src = "export function add(a: number, b: number): number {\n  return a + b;\n}\n";
    assert_eq!(
        skel(LanguageId::TypeScript, src),
        skel(LanguageId::TypeScript, src)
    );
}

#[test]
fn ts_functions_arrows_generics_generators() {
    let src = r#"
export function map<T>(items: T[], fn: (x: T) => T): T[] {
  const out = [];
  for (const i of items) out.push(fn(i));
  return out;
}
export const double = (x: number) => {
  return x * 2;
};
export function* range(n: number) {
  for (let i = 0; i < n; i++) yield i;
}
"#;
    let s = skel(LanguageId::TypeScript, src);
    assert_balanced(&s);
    assert!(
        s.contains("export function map<T>(items: T[], fn"),
        "signature kept:\n{s}"
    );
    assert!(
        s.contains("export const double = (x: number) =>"),
        "arrow kept:\n{s}"
    );
    assert!(
        s.contains("export function* range(n: number)"),
        "generator kept:\n{s}"
    );
    assert!(!s.contains("out.push(fn(i))"), "body removed:\n{s}");
    assert!(!s.contains("return x * 2"), "arrow body removed:\n{s}");
    assert!(!s.contains("yield i"), "generator body removed:\n{s}");
}

#[test]
fn ts_class_constructor_methods_interfaces() {
    let src = r#"
export class User {
  private name: string;
  constructor(name: string) {
    this.name = name;
  }
  getName(): string {
    return this.name.toUpperCase();
  }
  private static count = 0;
}
export interface Named {
  name: string;
  getName(): string;
}
"#;
    let s = skel(LanguageId::TypeScript, src);
    assert_balanced(&s);
    assert!(
        s.contains("constructor(name: string)"),
        "constructor kept:\n{s}"
    );
    assert!(s.contains("getName(): string"), "method kept:\n{s}");
    assert!(s.contains("export interface Named"), "interface kept:\n{s}");
    assert!(s.contains("private name: string"), "field kept:\n{s}");
    assert!(
        !s.contains("this.name = name"),
        "constructor body removed:\n{s}"
    );
    assert!(
        !s.contains("this.name.toUpperCase()"),
        "method body removed:\n{s}"
    );
}

#[test]
fn ts_decorators_and_nested_closures() {
    let src = r#"
@Injectable()
export class Service {
  handler() {
    const cb = (x: number) => {
      return x + 1;
    };
    return cb(1);
  }
}
"#;
    let s = skel(LanguageId::TypeScript, src);
    assert_balanced(&s);
    assert!(s.contains("@Injectable()"), "decorator preserved:\n{s}");
    assert!(s.contains("handler()"), "method kept:\n{s}");
    assert!(!s.contains("const cb"), "nested closure body removed:\n{s}");
    assert!(!s.contains("x + 1"), "nested body removed:\n{s}");
}

#[test]
fn python_classes_nested_async_decorators() {
    let src = r#"
@dataclass
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def dist(self):
        return (self.x ** 2 + self.y ** 2) ** 0.5

def outer():
    def inner():
        return 1
    return inner()

@route("/ping")
async def ping():
    await send_pong()
"#;
    let s = skel(LanguageId::Python, src);
    assert!(s.contains("def __init__(self, x, y)"), "ctor kept:\n{s}");
    assert!(s.contains("def dist(self)"), "method kept:\n{s}");
    assert!(s.contains("def outer()"), "outer kept:\n{s}");
    assert!(s.contains("@route(\"/ping\")"), "decorator kept:\n{s}");
    assert!(s.contains("async def ping()"), "async kept:\n{s}");
    assert!(!s.contains("self.x = x"), "ctor body removed:\n{s}");
    assert!(
        !s.contains("def inner()"),
        "nested function body removed:\n{s}"
    );
    assert!(!s.contains("await send_pong()"), "async body removed:\n{s}");
    assert!(s.contains("..."), "elision marker present:\n{s}");
}

#[test]
fn rust_structs_impls_traits_generics() {
    let src = r#"
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

pub trait Distance {
    fn distance(&self) -> f64 {
        0.0
    }
}

pub fn generic_max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
"#;
    let s = skel(LanguageId::Rust, src);
    assert_balanced(&s);
    assert!(s.contains("pub struct Point<T>"), "struct kept:\n{s}");
    assert!(s.contains("impl<T> Point<T>"), "impl kept:\n{s}");
    assert!(
        s.contains("pub fn new(x: T, y: T) -> Self"),
        "fn kept:\n{s}"
    );
    assert!(s.contains("pub trait Distance"), "trait kept:\n{s}");
    assert!(
        s.contains("fn distance(&self) -> f64"),
        "trait method kept:\n{s}"
    );
    assert!(
        s.contains("pub fn generic_max<T: PartialOrd>"),
        "generic fn kept:\n{s}"
    );
    assert!(!s.contains("Self { x, y }"), "impl body removed:\n{s}");
    assert!(!s.contains("a > b"), "fn body removed:\n{s}");
}

#[test]
fn go_functions_methods_interfaces() {
    let src = r#"
package main

type Greeter interface {
    Greet() string
}

type user struct {
    name string
}

func (u *user) Greet() string {
    return "hi " + u.name
}

func newUser(name string) *user {
    return &user{name: name}
}

var _ = func(x int) int {
    return x * 2
}
"#;
    let s = skel(LanguageId::Go, src);
    assert_balanced(&s);
    assert!(
        s.contains("func (u *user) Greet() string"),
        "method kept:\n{s}"
    );
    assert!(
        s.contains("func newUser(name string) *user"),
        "func kept:\n{s}"
    );
    assert!(s.contains("type Greeter interface"), "interface kept:\n{s}");
    assert!(s.contains("type user struct"), "struct kept:\n{s}");
    assert!(s.contains("name string"), "field kept:\n{s}");
    assert!(
        !s.contains("return \"hi \" + u.name"),
        "method body removed:\n{s}"
    );
    assert!(!s.contains("&user{name: name}"), "func body removed:\n{s}");
    assert!(!s.contains("x * 2"), "func literal body removed:\n{s}");
}

#[test]
fn skeleton_never_larger_than_source() {
    // A skeleton must never balloon the source (a cheap "no malformed output" guard).
    for (lang, src) in [
        (
            LanguageId::TypeScript,
            "export function a() {\n  return 1;\n}\n",
        ),
        (LanguageId::Python, "def a():\n    return 1\n"),
        (LanguageId::Rust, "fn a() -> i32 {\n    1\n}\n"),
        (
            LanguageId::Go,
            "package p\n\nfunc a() int {\n\treturn 1\n}\n",
        ),
    ] {
        let s = skeletonize(lang, src, "test.ext", Path::new(".")).unwrap();
        assert!(
            s.len() <= src.len() * 2 + 256,
            "skeleton unexpectedly large for {lang:?}:\n{s}"
        );
        assert!(!s.is_empty());
    }
}

/// `.tsx` files must use the TSX grammar (JSX-aware), not the plain
/// TypeScript grammar — otherwise JSX-heavy files fail to parse.
#[test]
fn tsx_files_parse_with_jsx_syntax() {
    let src = r#"
import { Link } from "next/link";

export default function Footer() {
  return (
    <footer className="border-t">
      <div className="flex items-center">
        <Link href="/docs">Documentation</Link>
      </div>
      <p className="font-mono text-xs">MIT licensed</p>
    </footer>
  );
}
"#;
    let s = skeletonize(
        LanguageId::TypeScript,
        src,
        "components/Footer.tsx",
        Path::new("."),
    )
    .unwrap();
    assert_balanced(&s);
    assert!(
        s.contains("export default function Footer()"),
        "component signature kept:\n{s}"
    );
}

#[test]
fn malformed_code_yields_bounded_declaration_skeleton() {
    for (lang, src) in [
        (
            LanguageId::TypeScript,
            "export function broken( {\n  return 1\n}\n",
        ),
        (LanguageId::Python, "def broken(:\n    return 1\n"),
        (LanguageId::Rust, "pub fn broken( {\n    return 1\n}\n"),
        (
            LanguageId::Go,
            "package main\n\nfunc broken( {\n\treturn 1\n}\n",
        ),
    ] {
        let s = skeletonize(lang, src, "x.ext", Path::new(".")).unwrap();
        assert_ne!(
            s, src,
            "malformed skeleton for {lang:?} must NOT dump the full source"
        );
        assert!(
            s.len() < src.len(),
            "malformed skeleton for {lang:?} must be smaller than source: {s:?}"
        );
        assert!(
            s.contains("broken"),
            "declaration header preserved for {lang:?}: {s:?}"
        );
    }
}

#[test]
fn malformed_code_skeleton_never_leaks_body_lines() {
    // A malformed file whose body carries secret-looking values: the fallback
    // skeleton must keep the declaration header only, never the body.
    let src = "function broken( {\n  const config = {\n    password: \"hunter2\",\n    api_key: \"sk-abc123\",\n  };\n";
    let s = skeletonize(LanguageId::TypeScript, src, "config.ts", Path::new(".")).unwrap();
    assert!(s.contains("function broken("), "header kept: {s:?}");
    assert!(
        !s.contains("hunter2") && !s.contains("sk-abc123"),
        "body lines must not leak: {s:?}"
    );
    assert!(s.len() < src.len(), "bounded: {s:?}");
}

#[test]
fn python_docstrings_preserved_in_skel() {
    let src = r#"
def documented(a: int) -> int:
    """Sums with one.

    Detailed explanation that is long.
    """
    return a + 1
"#;
    let s = skeletonize(LanguageId::Python, src, "x.py", Path::new(".")).unwrap();
    assert!(s.contains("\"\"\"Sums with one."), "docstring kept:\n{s}");
    assert!(!s.contains("return a + 1"), "implementation removed:\n{s}");
}

#[test]
fn python_main_block_reduced_but_guard_kept() {
    let src = r#"
import argparse

def run_server(port: int) -> None:
    print(f"listening on {port}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=8080)
    args = parser.parse_args()
    run_server(port=args.port)
"#;
    let s = skeletonize(LanguageId::Python, src, "x.py", Path::new(".")).unwrap();
    assert!(
        s.contains("if __name__ == \"__main__\":"),
        "main guard kept:\n{s}"
    );
    assert!(
        !s.contains("parser.add_argument"),
        "bootstrap body removed:\n{s}"
    );
    assert!(
        !s.contains("args = parser.parse_args()"),
        "body removed:\n{s}"
    );
    assert!(s.contains("..."), "elision present:\n{s}");
}

#[test]
fn python_method_placeholder_uses_body_indent() {
    let src = r#"
class Config:
    def load(self, path):
        """Load config from path."""
        import json
        return json.load(f)
"#;
    let s = skeletonize(LanguageId::Python, src, "x.py", Path::new(".")).unwrap();
    // The elision inside the method must be indented at the body depth (8
    // spaces), not at the class/def depth (4).
    assert!(
        s.contains("\n        ..."),
        "placeholder indented to body depth:\n{s}"
    );
    assert!(
        !s.contains("\n    ...\n"),
        "no placeholder at def depth:\n{s}"
    );
}

#[test]
fn typescript_accessors_object_methods_default_export() {
    let src = r#"
export class Bank {
  private _balance: number = 0;
  get balance(): number {
    return this._balance;
  }
  set balance(v: number) {
    if (v < 0) throw new Error("negative");
    this._balance = v;
  }
  static create(): Bank {
    return new Bank();
  }
}
export const obj = {
  greet(msg: string): string {
    return `${this.name}:${msg}`;
  },
};
"#;
    let s = skeletonize(LanguageId::TypeScript, src, "x.ts", Path::new(".")).unwrap();
    assert_balanced(&s);
    assert!(s.contains("get balance(): number"), "getter kept:\n{s}");
    assert!(s.contains("set balance(v: number)"), "setter kept:\n{s}");
    assert!(s.contains("static create(): Bank"), "static kept:\n{s}");
    assert!(
        s.contains("greet(msg: string): string"),
        "object method kept:\n{s}"
    );
    assert!(!s.contains("this._balance"), "getter body removed:\n{s}");
    assert!(!s.contains("new Bank()"), "static body removed:\n{s}");
}

#[test]
fn go_init_generics_embedding_variadic() {
    let src = r#"
package main

var registry = map[string]string{}

func init() {
    registry["a"] = "alpha"
}

type Builder[T any] struct {
    prefix string
    items  []T
}

func (b *Builder[T]) Add(item T) *Builder[T] {
    b.items = append(b.items, item)
    return b
}

func Variadic(args ...int) int {
    total := 0
    for _, a := range args {
        total += a
    }
    return total
}
"#;
    let s = skeletonize(LanguageId::Go, src, "x.go", Path::new(".")).unwrap();
    assert_balanced(&s);
    assert!(s.contains("func init()"), "init kept:\n{s}");
    assert!(
        s.contains("type Builder[T any] struct"),
        "generic struct kept:\n{s}"
    );
    assert!(
        s.contains("func (b *Builder[T]) Add(item T) *Builder[T]"),
        "generic method kept:\n{s}"
    );
    assert!(
        s.contains("func Variadic(args ...int) int"),
        "variadic kept:\n{s}"
    );
    assert!(!s.contains("registry[\"a\"]"), "init body removed:\n{s}");
    assert!(
        !s.contains("append(b.items, item)"),
        "method body removed:\n{s}"
    );
    assert!(!s.contains("total += a"), "variadic body removed:\n{s}");
}

#[test]
fn rust_generic_lifetime_impl_trait_kept() {
    let src = r#"
use std::fmt::Display;

pub fn max_by_key<'a, T: Ord, F: Fn(&'a T) -> T>(
    items: &'a [T],
    key: F,
) -> Option<&'a T> {
    items.iter().max_by_key(key)
}

pub struct Container<T> {
    inner: Vec<T>,
}

impl<T: Display + Clone> Container<T> {
    pub fn to_strings(&self) -> Vec<String> {
        self.inner.iter().map(|x| x.to_string()).collect()
    }
}
"#;
    let s = skeletonize(LanguageId::Rust, src, "x.rs", Path::new(".")).unwrap();
    assert_balanced(&s);
    assert!(
        s.contains("pub fn max_by_key<'a, T: Ord, F: Fn(&'a T) -> T>("),
        "generic fn kept:\n{s}"
    );
    assert!(s.contains("items: &'a [T],"), "fn arg kept:\n{s}");
    assert!(
        s.contains("pub struct Container<T>"),
        "generic struct kept:\n{s}"
    );
    assert!(
        s.contains("impl<T: Display + Clone> Container<T>"),
        "generic impl kept:\n{s}"
    );
    assert!(!s.contains("max_by_key(key)"), "fn body removed:\n{s}");
    assert!(!s.contains("x.to_string()"), "method body removed:\n{s}");
}
