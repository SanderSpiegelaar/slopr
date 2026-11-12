# Tickets

Each ticket is one PR. The docs under `docs/reference/` are the spec. A ticket says which part of the spec to build, in which order, and how to prove it.

## Picking a ticket

Tickets live in `wave-NN/` folders. Every ticket in a wave can be built in parallel once all earlier waves are merged. A ticket is ready as soon as its own `depends_on` are merged, which can be before its whole wave is.

A ticket's wave is its longest dependency chain, so the folder is derived from `depends_on`. After changing `depends_on`, run `python3 scripts/check-tickets.py --place` to move files. The check fails while a file is in the wrong folder.

## Frontmatter

| Key | Meaning |
|---|---|
| `id` | File name without `.md`, inside its wave folder. `D-*` decision, `S00` spike, `T*` core, `F*` fallow, `C*` CI, `R-*` v0 rule family, `G*` phase gate, `V*` v1, `E*` v2 epic. |
| `kind` | `decision`, `spike`, `code`, `gate` or `epic`. |
| `depends_on` | Tickets that must be merged first. |
| `docs` | `file#anchor` sections to read. Read only these. An anchor that does not exist yet is created by a dependency's `creates`. |
| `touches` | Files, or directories ending in `/`, that the PR may change. |
| `verify` | A bash command run from the repo root. It fails before the ticket and passes after it. |
| `covers` | What the ticket closes: `rule:<id>` from the rule catalog, `roadmap:<clause>`, `gap:<id>`. Each appears in exactly one ticket. |
| `creates` | Doc headings a decision ticket adds. |
| `signoff` | `user` means Sander approves the PR before merge. |

`scripts/tests-ran.sh <target> <filter>` runs `cargo test --test <target>` (or `--bin slopr` for `bin`) and also fails when no test matched the filter.

## Rules for implementers

- Change only paths in `touches`. If the work needs another path, stop and report it. Do not widen the ticket.
- A decision ticket writes the decision in its Scope into the named doc, as prose under the named heading. It changes no code.
- Test names in acceptance criteria are exact. `verify` filters on them.
- Before handing back, run the four commands in `AGENTS.md` and the ticket's `verify`.

## Conflict plan

Two tickets that touch the same path are always ordered by `depends_on`. `scripts/check-tickets.py` fails otherwise. Three choices keep most work parallel.

- **`RULES` table.** `T01` creates `src/rules/mod.rs` with `FAMILIES: &[&[Rule]]` and one empty `pub const RULES: &[Rule]` per v0 family module. A family ticket fills only its own module, so no two rule tickets share a file. `V04` adds the v1 family modules the same way.
- **Shared helpers before rules.** `T01` puts the comment, string and test-context helpers in `source.rs` and adds every v0 crate to `Cargo.toml`, so rule tickets never touch those files.
- **Notices before ports.** `D-SOURCES` writes `THIRD_PARTY_NOTICES.md` for every source a rule will port, so rule tickets never touch it.

`src/main.rs` and `tests/cli.rs` sit on one ordered chain of core tickets. Rule tickets never touch them.

## Gaps

Each gap is a fact an implementer would otherwise guess. Its decision ticket lists it in `covers`.

| ID | Gap |
|---|---|
| D01 | `RuleSettings` is not defined, and only `issue_pattern` is documented. |
| D02 | Config discovery, severity values, override precedence, glob base, `exclude` with gitignore, root with several PATHS. |
| D03 | `Rule` has no default-on flag, yet `comment/obvious` is off by default. |
| D04 | File model: no JS grammar, `langs: []` versus a required `tree`, extension map, binary and non-UTF-8 files, parse errors. |
| D05 | `is_test` detection per language, including Rust `#[cfg(test)]` modules. |
| D06 | Span basis: 0- or 1-based, byte or char columns. |
| D07 | Fingerprint normalization, a hash crate not in the dependency list, identical-line collisions. |
| D08a | Language coverage and node patterns for each v0 rule. |
| D08b | Language coverage and node patterns for each v1 rule. |
| D09 | Ported lists have no pinned upstream commit or file path. Secret regexes and redaction are open. |
| D10 | Suppression matching: comment syntax, next line, several IDs, fallow findings, file directive, invalid directive. |
| D11 | Baseline file format, missing file, stale entries. |
| D12 | Exit gate values, precedence of exit 2 and 3, per-file read errors. |
| D13 | Output contracts: human layout, json `summary` and `tools`, SARIF metadata, sort order. |
| D14 | The SARIF schema test needs a validator crate not in the dependency list. |
| D15 | fallow details: root, workspace detection, URI normalization, multi-location results, fingerprints, `args`, overlap flag. |
| D16 | Resolved in the roadmap: `init` and `rules` in v0, `explain` and release files in v1. |
| D17 | `slopr explain` has no source for rule sources or fixture examples. |
| D18 | Diff mode: changed-line semantics, `--staged` content, untracked files, fallow in diff mode, `pr/*` locations. |
| D19 | Native complexity formulas and duplication windows. |
| D20 | Which rules extend to Go. |
| D21 | FP corpus: script, pinned commits, line counting. |
| D22 | No CI workflow is specified for tests and the fallow contract jobs. |
| D23 | `fallow schema` output format for `slopr rules --all`. |
| D24 | `text/invisible-unicode` would flag a leading BOM. |
| D25 | `dup/exact-block` (cross-file) and `pr/*` (diff-level) do not fit the per-file `check` signature. |
