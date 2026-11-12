# slopr plan

Working name: `slopr` (placeholder, rename before publishing). Plan date: 2026-11-11. Author: Recon (read-only analysis). Nothing in this plan has been built or run.

## Goal and bottom line

Goal: one Rust binary that flags the code-quality problems AI coding agents leave behind, combining the deterministic checks found in the source repos below, and exposing every fallow check for TypeScript and JavaScript projects.

Bottom line:

- Build a single-crate Rust CLI. v0 ships 13 native, deterministic rules on tree-sitter (comment slop, swallowed errors, stubs, tautological tests, debug leftovers, unsafe TS assertions, invisible Unicode, hardcoded secrets) plus fallow passthrough, with human, JSON and SARIF output and CI exit codes.
- fallow runs as a separately installed subprocess. See the [decision record](research/decisions.md#decision-record-2026-11-11).
- Do not re-implement what fallow already does well for JS/TS (dead code, unlisted and unresolved imports, duplication, complexity, cycles, boundaries, weakened tests). Native rules target slop signals fallow does not cover and languages fallow does not parse.
- LLM or embedding checks are optional and come last (phase v2).

## Assumptions (no blocking question was asked)

- Target implementation language is Rust (stable toolchain).
- Open-source friendly licensing is wanted. Recommended project license: `MIT OR Apache-2.0`, which allows porting MIT and Apache-2.0 sources with attribution.
- Must-have: deterministic local checks and fallow passthrough. Nice-to-have: LLM or embedding checks.
- No telemetry, no network access during `check`.
- v0 languages: TypeScript, TSX, JavaScript, Python, Rust. Go joins in v1.

## Task router

Read the files listed for your task. Do not load the others.

| Task | Files |
|---|---|
| Phase 0 spike | [dependencies](reference/dependencies.md), [fallow](reference/fallow.md), [roadmap](roadmap.md) |
| Scaffold the core pipeline (walk, source, finding, main) | [architecture](reference/architecture.md), [dependencies](reference/dependencies.md), [cli](reference/cli.md), [config](reference/config.md) |
| Implement or add a rule | [rules](reference/rules.md), [architecture](reference/architecture.md), [config](reference/config.md), [testing](reference/testing.md), [licensing](reference/licensing.md). When porting a phrase list or pattern, also [sources](research/sources.md#per-repo-notes-code-read-paths-cited). |
| fallow integration | [fallow](reference/fallow.md), [architecture](reference/architecture.md), [cli](reference/cli.md), [config](reference/config.md), [testing](reference/testing.md) |
| CLI, output formats, exit codes | [architecture](reference/architecture.md), [cli](reference/cli.md) |
| Config, suppressions, baseline | [architecture](reference/architecture.md), [cli](reference/cli.md), [config](reference/config.md) |
| Diff mode (`--changed-since`, `--staged`, `pr/*` rules) | [rules](reference/rules.md), [fallow](reference/fallow.md), [cli](reference/cli.md), [roadmap](roadmap.md) |
| Test infrastructure (harness, snapshots, FP corpus) | [architecture](reference/architecture.md), [cli](reference/cli.md), [testing](reference/testing.md) |
| CI, pre-commit, agent skill | [cli](reference/cli.md) |
| Release (licenses, notices, naming) | [dependencies](reference/dependencies.md), [licensing](reference/licensing.md), [roadmap](roadmap.md) |
| v2 semantic duplicates | [rules](reference/rules.md), [licensing](reference/licensing.md), [roadmap](roadmap.md), [sources](research/sources.md) |
| Implement a ticket | [tickets](tickets/README.md), the ticket file, and the files in its `docs` list |
| Revisit a decision or a source | [roadmap](roadmap.md), [sources](research/sources.md), [decisions](research/decisions.md) |
