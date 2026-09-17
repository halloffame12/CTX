# CTX — Skeleton Engine Test + Fix Report

## Summary

Built a 37-file skeleton test corpus covering every supported language
(TypeScript, Python, Rust, Go) across tiny files, large files, deeply nested
code, classes, functions, interfaces, types, structs, enums, generics,
decorators, comments, malformed code, and generated code. Verified against a
ground-truth harness that skeleton output (a) preserves imports, exports,
signatures, types, symbol hierarchy and architecture, and (b) reduces
implementation bodies. 2 engine bugs were found and fixed; 7 regression tests
were added. All 90 tests pass in debug and release; fmt/clippy clean.

## Corpus (`tests/fixtures/skeleton/`, 37 files)

Per language: `tiny`, `module` (imports/exports/classes/interfaces/types/
enums/structs/traits), `nested` (deep nesting + closures + decorators),
`generated` (DO NOT EDIT style), `malformed` (broken syntax), `large` (~120
functions), plus language-specific edge cases:

- **ts**: `accessors` (getters/setters/static/object methods), `default.tsx`
  (default export + JSX), `classes.ts`, `generics` (generics + type aliases +
  interfaces), `realistic.ts` (real Supabase/async code).
- **py**: `classes` (nested class + property getter/setter), `types` (typing
  generics), `mainblock` (`if __name__ == "__main__"`), `docstrings`
  (module/class/function docstrings).
- **rs**: `generics` (lifetimes + generics + `impl Trait`), `impls` (trait
  default method, const/static, impl blocks).
- **go**: `init` (init() + Go 1.18 generics + variadic), `embed` (struct
  embedding + interfaces).

## Baseline

The engine already reduced function bodies well in the common case
(signatures preserved, bodies collapsed, balance maintained). The harness
found **2 real bugs** in edge cases and 0 correct-but-under-tested paths that
needed hardening.

## Bugs found and fixed

### Bug 1 — Malformed code produced structurally corrupt skeletons

**Reproduce**: a malformed brace-language file (missing `)`, unbalanced `{`)
was parsed by tree-sitter into a tree with `ERROR`/`MISSING` nodes. The
skeleton spliced body interiors anyway, producing unbalanced braces —
e.g. `pub fn broken( { /* ... implementation hidden ... */ }` for Rust. This
is *worse* than the input: a consumer that re-parses the skeleton gets a
different, corrupted file.

**Fix** (`src/parser/util.rs`, `src/parser/python.rs`): before splicing, check
`has_errors(&root)`. If the parse tree has errors, return the source unchanged
(graceful degradation). Malformed files now produce a skeleton that is
byte-identical to the source — never a half-spliced misrepresentation.

**Regression test**: `malformed_code_degrades_to_source_unchanged` (all 4
languages assert skeleton == source).

### Bug 2 — Python `if __name__ == "__main__"` bodies were not reduced

**Reproduce**: the module entry-point bootstrap block
(`argparse` setup, `run_server(port=args.port)`) stayed verbatim in the
skeleton. That is implementation detail an AI does not need — it masks the
file's real architecture (imports, constants, function signatures).

**Fix** (`src/parser/python.rs`): detect `if_statement` whose condition is
`__name__ == "__main__"` and reduce its `consequence` block to `...`, keeping
the guard line (which marks the entry point).

**Regression test**: `python_main_block_reduced_but_guard_kept`.

### Bug 3 — Python function/method docstrings were stripped

**Reproduce**: `def documented(a: int) -> int:` with a multi-line docstring
lost the docstring entirely when the body was reduced. Docstrings are
documentation, not implementation — an AI needs them to understand what the
function does. (Module-level and class-level docstrings were already kept.)

**Fix** (`src/parser/python.rs`): when reducing a function/async-function
body, if the body's first statement is a string (a docstring), keep the
docstring and reduce only the remainder.

**Regression test**: `python_docstrings_preserved_in_skel`.

### Bug 4 — Python placeholder indentation was wrong inside methods

**Reproduce**: a method body placeholder `...` was emitted at 4-space indent
instead of the body's 8-space indent, because indent was measured from the
block start byte (which lands on the newline after `:`) rather than the first
statement.

**Fix** (`src/parser/python.rs`): `first_line_indent` now walks back to the
start of the line containing the given byte before measuring indentation, and
callers measure from the body's first statement.

**Regression test**: `python_method_placeholder_uses_body_indent`.

## What was verified (and passed without changes)

- **TS**: getters/setters, static methods, object literal methods, default
  exports, JSX (`default.tsx`), generics (`<T, K extends keyof T>`), type
  aliases, interfaces, arrow functions, generators, decorators, nested
  closures, generated-code comments.
- **Py**: nested classes, `@property`/setter, async functions, typing
  generics, decorators, docstrings (module/class kept, function now kept).
- **Rust**: lifetimes + generics, `impl Trait`, trait default method bodies,
  `impl` blocks, `const`/`static`, closures, structs/enums/fields.
- **Go**: `init()`, `main()`, Go generics (`Builder[T any]`), variadics,
  struct embedding, interfaces, method receivers, func literals.
- **Reduction quality**: a realistic 1.9 KB Supabase/async TS file reduces to
  497 bytes (74% reduction) with all imports and signatures preserved.
- **Balance**: every brace-language skeleton is brace/paren/bracket balanced.
- **Determinism**: `deterministic_output` still passes.
- **Skeleton never larger than source** guard still holds.

## Harness results (final)

```
========== TS ==========  11 files OK (tiny, module, nested, generated,
  malformed-unchanged, large, accessors, default.tsx, classes, generics,
  realistic)
========== PY ==========  10 files OK (…, classes, types, mainblock, docstrings)
========== RS ==========   8 files OK (…, generics, impls)
========== GO ==========   8 files OK (…, init, embed)

keep-checks passed=198 reduce-checks passed=89 failures=0
```

Every fixture asserts: required signatures/imports/exports/types/decorators/
comments are **present**, and implementation statements (assignments,
returns, awaits, map/filter bodies, DB calls) are **absent**.

## Regression tests added (`tests/skeleton.rs`, 7 new)

1. `malformed_code_degrades_to_source_unchanged` — all 4 languages.
2. `python_docstrings_preserved_in_skel`
3. `python_main_block_reduced_but_guard_kept`
4. `python_method_placeholder_uses_body_indent`
5. `typescript_accessors_object_methods_default_export`
6. `go_init_generics_embedding_variadic`
7. `rust_generic_lifetime_impl_trait_kept`

## Quality gates

- `cargo test`: **90 passed** (21 unit + 53 integration + 16 skeleton) in
  both debug and release.
- `cargo fmt --check`: clean.
- `cargo clippy --all-targets`: 0 warnings.
- No cross-language regressions: the existing 9 skeleton tests, all 53
  integration tests, and 21 unit tests still pass.

## Files changed

- `src/parser/util.rs` — malformed-tree detection before body splicing in
  `skeleton_brace_wrapped`.
- `src/parser/python.rs` — graceful degradation on parse errors; `__main__`
  block reduction; docstring preservation; line-aware indent measurement.
- `tests/skeleton.rs` — 7 new regression tests.
- `tests/fixtures/skeleton/` — permanent 37-file skeleton corpus.

## Final requirement

An AI reading a skeleton now sees the file's architecture — imports, exports,
signatures, types, interfaces/classes/structs/enums, method hierarchies, and
key markers like decorators, `if __name__ == "__main__"`, and function
docstrings — without any implementation detail (bodies are a single elision),
and malformed or generated files degrade safely instead of corrupting output.