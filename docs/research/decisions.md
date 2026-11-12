# Decisions

## fallow integration: options compared

| Option | Stability | Version pinning | License | Build complexity | Startup cost | Verdict |
|---|---|---|---|---|---|---|
| Subprocess: `fallow <cmd> --format sarif` (and JSON where needed) | CLI and JSON are semver-covered; SARIF fields required by the spec are stable | Check `fallow --version` against a semver range; users upgrade fallow on their own schedule | MIT, no linking | None for slopr; user installs fallow via npm, cargo, Homebrew, Docker or release binary | One process per command; fallow keeps a cache in `.fallow/` | Recommended |
| Link `fallow-api` or `fallow-engine` 3.42 crates | Not semver-covered; may break in any minor | Requires `=3.42.x` exact pin, re-port on every minor (5 releases in 10 days) | MIT | MSRV 1.97, edition 2024, oxc 0.157, exact pins on `lightningcss` alpha and `cssparser`; heavy compile; conflicts if slopr adds its own oxc | None | Rejected |
| NAPI `@fallow-cli/fallow-node` | "API is new", minor can break | npm | MIT | Embeds a Node runtime in a Rust CLI | Node boot | Rejected |
| fallow MCP server | MCP tools wrap the CLI JSON | | MIT | MCP client in slopr | Extra handshake | Rejected for slopr; agents can call fallow MCP directly |

Recommendation: run a separately installed fallow binary as a subprocess, accept versions `>=3.42, <4`, read SARIF for findings and `fallow schema` for the list of checks.

## Decision record (2026-11-11)

Decision: slopr integrates fallow as a subprocess and reads its SARIF output. Supported fallow versions are `>=3.42, <4`. The user installs fallow separately (npm, cargo, Homebrew, Docker or release binary). slopr checks the fallow version at startup and, when fallow is missing or out of range, stops with a clear error that names the found version (if any), the required range, and an install hint such as `npm install -D fallow@^3.42` or `cargo install fallow-cli --version ^3.42`. Approved by Sander on 2026-11-11.

- Integrate fallow by running a separately installed, version-checked `fallow` binary as a subprocess and ingesting its SARIF output. Do not link the fallow Rust crates and do not use the NAPI bindings. Reason: fallow documents its CLI and JSON output as semver-stable, and documents every Rust crate (including `fallow-api`) as "not supported external semver APIs" that "may change in a minor release" (`fallow-rs/fallow:docs/backwards-compatibility.md`, section "What is not covered").

Rejected: shipping slopr as one self-contained binary with fallow linked in. It would need an exact pin on fallow's Rust crates, which are not semver-stable, and a re-port on every fallow minor release.

## Crates not used

Not used, with reason: `serde-sarif` 0.8.0 (last release 2025-05, and slopr needs raw passthrough of fallow runs anyway; a 60-line struct set is enough); `ast-grep-core` 0.51.2 (good option if rules move to declarative patterns; not needed for v0 Rust-function rules); `oxc_parser` (fallow already covers JS/TS semantics; tree-sitter gives one API across languages); `rmcp` 3.9.0 (MCP server deferred to v2).

## Ticket plan (2026-11-12)

Approved by Sander on 2026-11-12.

- Tickets live as markdown in [docs/tickets](../tickets/README.md). `scripts/check-tickets.py` checks them against the roadmap, the rule catalog and the gap list.
- One ticket per rule family (one `src/rules/<family>.rs` module), not one per rule.
- Full tickets through v1. v2 items are epics.
- Gap tickets apply the documented default. Tickets marked `signoff: user` need Sander's approval on the PR.
- `slopr init` and `slopr rules` ship in v0. `slopr explain` and the release files (`LICENSE-MIT`, `LICENSE-APACHE`, `THIRD_PARTY_NOTICES.md`) ship in v1. Renaming slopr stays open until publishing.

## Core types (2026-11-12)

- FNV-1a replaces sha256 for the fingerprint. It needs no new crate and its output is stable.
- `via_fallow_for_js` is removed, because no v0 or v1 rule needs it and `langs` already says it.

## SARIF validation (2026-11-12)

- Field assertions in `tests/cli.rs` replace the SARIF schema test, because no schema validator crate is in the approved dependency list. See [SARIF checks](../reference/testing.md#sarif-checks).

## fallow output format (2026-11-12)

- slopr reads fallow's JSON, not its SARIF. The Phase 0 SARIF drops the second file of a cycle, the link between clone instances, the cognitive score, every end position, and whether an audit finding is new. See [Phase 0 findings](../reference/fallow.md#phase-0-findings).
- slopr's SARIF output builds the fallow run from the mapped findings instead of appending fallow's runs. See [Run details](../reference/fallow.md#run-details).
- A full scan runs one bare `fallow --only <L>` for `dead-code`, `dupes`, `health` and `architecture`, plus one `fallow security` and one `fallow flags` run when listed. The bare envelope has no security or flag section. See [Behavior](../reference/fallow.md#behavior).
- Severity comes from each finding's `effective_severity`, which reflects `.fallowrc` overrides. Diff mode keeps only audit findings with `introduced: true`. JSON columns are 0-based UTF-8 byte offsets, and `end_col` is exclusive. See [Field mapping](../reference/fallow.md#field-mapping).
- This replaces the SARIF parts of the 2026-11-11 record.
