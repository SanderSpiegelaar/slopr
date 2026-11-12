---
id: D-ARCH
title: Decide the core type gaps in architecture.md
kind: decision
depends_on: []
docs: [docs/reference/architecture.md#core-types, docs/reference/cli.md#cli, docs/reference/rules.md#merged-rule-catalog]
touches: [docs/reference/architecture.md, docs/research/decisions.md, docs/roadmap.md]
creates: [docs/reference/architecture.md#file-model, docs/reference/architecture.md#test-files, docs/reference/architecture.md#spans, docs/reference/architecture.md#fingerprint, docs/reference/architecture.md#rule-kinds, docs/reference/architecture.md#explain-data]
covers: [gap:D03, gap:D04, gap:D05, gap:D06, gap:D07, gap:D17, gap:D25]
signoff: user
verify: for h in "File model" "Test files" "Spans" "Fingerprint" "Rule kinds" "Explain data"; do grep -qx "## $h" docs/reference/architecture.md || exit 1; done; grep -q 'default_on: bool' docs/reference/architecture.md && ! grep -q via_fallow_for_js docs/reference/architecture.md docs/roadmap.md
---

## Goal

Write the core type decisions into `architecture.md`, so the skeleton and every rule ticket build against one shape.

## Scope

Edit `## Core types`:

- `Rule` loses `via_fallow_for_js` and gains `default_on: bool`. Its `check` field becomes `check: Check`.
- `SourceFile` becomes `{ path: String, text: String, parsed: Option<Parsed>, is_test: bool }`, with `pub struct Parsed { pub lang: Lang, pub tree: tree_sitter::Tree }`.
- The `fingerprint` comment says `hex(fnv1a64(...))`, 16 lowercase hex chars.
- One line per file: `finding.rs` owns `Severity`, `Span`, `Origin`, `Finding`. `source.rs` owns `Lang`, `Parsed`, `SourceFile`. `rules/mod.rs` owns `Rule`, `Check`, `Hit`, `FAMILIES`. `config.rs` owns `RuleSettings`.

Add these sections after `## Core types`:

- `## File model`. Extension map: `.ts .mts .cts` TypeScript, `.tsx` Tsx, `.js .mjs .cjs .jsx` JavaScript, `.py .pyi` Python, `.rs` Rust, `.go` Go (v1). JavaScript is parsed with the TSX grammar, because no JS grammar crate is in the dependency list. Every other file is a text file with `parsed: None`. A file is skipped, silently, when it is over 1 MiB, is not valid UTF-8, or has a NUL byte in its first 8 KiB. A tree with ERROR nodes is still used, and rules ignore ERROR subtrees. A rule with `langs: &[]` runs on every file, parsed or not. A rule with languages runs only on parsed files of those languages.
- `## Test files`. `is_test` is true when the scan-relative path matches any of `**/test/**`, `**/tests/**`, `**/__tests__/**`, `**/*.test.*`, `**/*.spec.*`, `**/test_*.py`, `**/*_test.py`, `**/conftest.py`, `**/*_test.go`. Rust code inside a `mod` item with `#[cfg(test)]`, or a fn with `#[test]`, is test code too. `SourceFile::in_test(node)` returns `is_test`, or for Rust whether such an ancestor exists.
- `## Spans`. Lines and columns are 1-based. Columns count Unicode scalar values, not bytes. `end_line` is inclusive and `end_col` is exclusive, as in SARIF. `source.rs` converts tree-sitter byte positions through the line index.
- `## Fingerprint`. Snippet: the lines `start_line..=end_line`, each trimmed, inner whitespace runs collapsed to one space, joined with `\n`. Occurrence: the 0-based count of earlier findings in the same file with the same rule and snippet, in report order. Fingerprint: FNV-1a 64 over `rule \0 path \0 snippet \0 occurrence`, printed as 16 lowercase hex chars. FNV-1a is ten lines of std-only code and is stable across Rust releases. `std::hash::DefaultHasher` is not.
- `## Rule kinds`. `pub enum Check { File(fn(&SourceFile, &RuleSettings, &mut Vec<Hit>)), Project(fn(&[SourceFile], &RuleSettings, &mut Vec<(usize, Hit)>)), Diff(fn(&Diff, &[SourceFile], &RuleSettings, &mut Vec<(usize, Hit)>)) }`. The `usize` indexes the `SourceFile` slice. `File` covers every v0 rule. `Project` is for `dup/exact-block`. `Diff` is for the `pr/*` rules, with `Diff` from `diff.rs`. `T01` builds `File` only. `V04` adds the other two variants.
- `## Explain data`. `build.rs` (std only) writes `$OUT_DIR/explain.rs`, a static table that maps each fixture dir under `tests/fixtures/rules/` to `include_str!` of its first `bad.*` file in name order. `slopr explain` also embeds `docs/reference/rules.md` with `include_str!` and prints the Sources cell of the rule's row. `Rule` gets no `sources` field. The catalog stays the single source.

In `docs/roadmap.md` under Risks, replace the double-reporting mitigation with "rules that fallow covers for JS/TS leave those languages out of `langs`".

Append `## Core types (2026-11-12)` to `docs/research/decisions.md`: FNV-1a replaces sha256 (no new crate, stable output). `via_fallow_for_js` is removed, because no v0 or v1 rule needs it and `langs` already says it.

## Out of scope

- Any Rust code.
- Rule detection details. Those belong to `D-RULES-V0` and `D-RULES-V1`.

## Acceptance criteria

- All six sections exist with the content above.
- `architecture.md` and `roadmap.md` no longer mention `via_fallow_for_js`.
- `decisions.md` has the new record.
- `python3 scripts/check-tickets.py` passes.
