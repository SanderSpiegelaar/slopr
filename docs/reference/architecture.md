# Architecture

## Shape

One binary crate, modules not crates. Split into a workspace only when compile time demands it.

Data flow: config, walk, parse, run native rules, run fallow (in parallel with native rules on a separate thread), apply suppressions and baseline, sort, report, exit code.

## Folder tree

```
slopr/
  Cargo.toml
  LICENSE-MIT
  LICENSE-APACHE
  THIRD_PARTY_NOTICES.md
  src/
    main.rs            clap CLI, exit codes
    config.rs          slopr.toml model and loading
    walk.rs            ignore::WalkBuilder, language detection by extension
    source.rs          SourceFile: text, tree-sitter Tree, line index
    finding.rs         Finding, Severity, Span, Fingerprint
    rules/
      mod.rs           FAMILIES table, run_rules
      comments.rs
      errors.rs
      stubs.rs
      tests.rs
      debug.rs
      types.rs
      text.rs          invisible unicode, secrets
      phrases.rs       phrase lists, each list cites its source and license
    fallow.rs          locate, version check, run, JSON to Finding
    diff.rs            git diff --unified=0 parsing for --changed-since and --staged
    suppress.rs        inline directives and baseline file
    report/
      human.rs
      json.rs
      sarif.rs
  skill/
    SKILL.md
  tests/
    rules.rs           fixture harness
    cli.rs             end-to-end snapshots
    fallow_contract.rs
    fixtures/
      rules/<rule-id>/bad.<ext>
      rules/<rule-id>/good.<ext>
      repos/ts-app/    package.json, planted fallow issues
      repos/py-lib/
      repos/rust-crate/
```

## Core types

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity { Info, Warn, Error }

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang { TypeScript, Tsx, JavaScript, Python, Rust, Go }

#[derive(Clone, Copy, Serialize)]
pub struct Span { pub start_line: u32, pub start_col: u32, pub end_line: u32, pub end_col: u32 }

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Origin { Native, Fallow { version: String, command: String } }

#[derive(Clone, Serialize)]
pub struct Finding {
    pub rule: String,            // "error/swallowed" or "fallow/unused-exports"
    pub severity: Severity,
    pub path: String,            // repo-relative, forward slashes
    pub span: Span,
    pub message: String,
    pub fingerprint: String,     // hex(fnv1a64(...)), 16 lowercase hex chars
    pub origin: Origin,
}

pub struct Parsed { pub lang: Lang, pub tree: tree_sitter::Tree }

pub struct SourceFile { pub path: String, pub text: String, pub parsed: Option<Parsed>, pub is_test: bool }

pub struct Hit { pub span: Span, pub message: String }

pub struct Rule {
    pub id: &'static str,
    pub summary: &'static str,
    pub default_severity: Severity,
    pub langs: &'static [Lang],     // empty = all files, including unparsed text
    pub default_on: bool,
    pub check: Check,
}

pub static FAMILIES: &[&[Rule]] = &[ /* one RULES slice per family module */ ];
```

Each type lives in one file.

- `finding.rs` owns `Severity`, `Span`, `Origin`, `Finding`.
- `source.rs` owns `Lang`, `Parsed`, `SourceFile`.
- `rules/mod.rs` owns `Rule`, `Check`, `Hit`, `FAMILIES`.
- `config.rs` owns `RuleSettings`.

Rules are a static table of plain functions. No trait objects, no plugin loading, no rule DSL in v0. Adding a rule is one function, one table row, one fixture directory.

## File model

`walk.rs` maps extensions to languages.

| Extensions | Lang |
|---|---|
| `.ts` `.mts` `.cts` | TypeScript |
| `.tsx` | Tsx |
| `.js` `.mjs` `.cjs` `.jsx` | JavaScript |
| `.py` `.pyi` | Python |
| `.rs` | Rust |
| `.go` | Go (v1) |

JavaScript is parsed with the TSX grammar, because no JS grammar crate is in the dependency list. Every other file is a text file with `parsed: None`.

A file is skipped, silently, when it is over 1 MiB, is not valid UTF-8, or has a NUL byte in its first 8 KiB.

A tree with ERROR nodes is still used. Rules ignore ERROR subtrees.

A rule with `langs: &[]` runs on every file, parsed or not. A rule with languages runs only on parsed files of those languages.

## Test files

`is_test` is true when the scan-relative path matches any of these globs.

- `**/test/**`
- `**/tests/**`
- `**/__tests__/**`
- `**/*.test.*`
- `**/*.spec.*`
- `**/test_*.py`
- `**/*_test.py`
- `**/conftest.py`
- `**/*_test.go`

Rust code inside a `mod` item with `#[cfg(test)]`, or inside a fn with `#[test]`, is test code too. `SourceFile::in_test(node)` returns `is_test`. For Rust it also returns true when such an ancestor exists.

## Spans

Lines and columns are 1-based. Columns count Unicode scalar values, not bytes. `end_line` is inclusive and `end_col` is exclusive, as in SARIF. `source.rs` converts tree-sitter byte positions through the line index.

## Fingerprint

The fingerprint uses the text of the flagged lines, not line numbers, so it survives line shifts. fallow `finding_id` and vibecheck `src/fingerprint.ts` use the same idea.

- Snippet: the lines `start_line..=end_line`, each trimmed, inner whitespace runs collapsed to one space, joined with `\n`.
- Occurrence: the 0-based count of earlier findings in the same file with the same rule and snippet, in report order.
- Fingerprint: FNV-1a 64 over `rule \0 path \0 snippet \0 occurrence`, printed as 16 lowercase hex chars.

FNV-1a is ten lines of std-only code and is stable across Rust releases. `std::hash::DefaultHasher` is not.

## Rule kinds

```rust
pub enum Check {
    File(fn(&SourceFile, &RuleSettings, &mut Vec<Hit>)),
    Project(fn(&[SourceFile], &RuleSettings, &mut Vec<(usize, Hit)>)),
    Diff(fn(&Diff, &[SourceFile], &RuleSettings, &mut Vec<(usize, Hit)>)),
}
```

The `usize` indexes the `SourceFile` slice. `File` covers every v0 rule. `Project` is for `dup/exact-block`. `Diff` is for the `pr/*` rules, with `Diff` from `diff.rs`. `T01` builds `File` only. `V04` adds the other two variants.

## Explain data

`build.rs` (std only) writes `$OUT_DIR/explain.rs`. It is a static table that maps each fixture dir under `tests/fixtures/rules/` to `include_str!` of its first `bad.*` file in name order.

`slopr explain` also embeds `docs/reference/rules.md` with `include_str!` and prints the Sources cell of the rule's row. `Rule` gets no `sources` field. The catalog stays the single source.
