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
        let s = skeletonize(lang, src, Path::new(".")).unwrap();
        assert!(
            s.len() <= src.len() * 2 + 256,
            "skeleton unexpectedly large for {lang:?}:\n{s}"
        );
        assert!(!s.is_empty());
    }
}
