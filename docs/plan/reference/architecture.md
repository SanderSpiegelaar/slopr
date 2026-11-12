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
      mod.rs           RULES table, run_rules
      comments.rs
      errors.rs
      stubs.rs
      tests.rs
      debug.rs
      types.rs
      text.rs          invisible unicode, secrets
      phrases.rs       phrase lists, each list cites its source and license
    fallow.rs          locate, version check, run, SARIF to Finding
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
    pub fingerprint: String,     // hex(sha256(rule, path, normalized snippet))[..16]
    pub origin: Origin,
}

pub struct SourceFile { pub path: String, pub lang: Lang, pub text: String, pub tree: tree_sitter::Tree, pub is_test: bool }

pub struct Hit { pub span: Span, pub message: String }

pub struct Rule {
    pub id: &'static str,
    pub summary: &'static str,
    pub default_severity: Severity,
    pub langs: &'static [Lang],     // empty = all files, including unparsed text
    pub via_fallow_for_js: bool,    // skip on JS/TS when fallow ran
    pub check: fn(&SourceFile, &RuleSettings, &mut Vec<Hit>),
}

pub static RULES: &[Rule] = &[ /* one entry per native rule */ ];
```

Rules are a static table of plain functions. No trait objects, no plugin loading, no rule DSL in v0. Adding a rule is one function, one table row, one fixture directory.

Fingerprint uses the trimmed text of the flagged lines, not line numbers, so it survives line shifts (same idea as fallow `finding_id` and vibecheck `src/fingerprint.ts`).
