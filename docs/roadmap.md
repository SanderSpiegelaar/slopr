# Roadmap

One bullet is one clause. `scripts/check-tickets.py` maps each clause to exactly one ticket in [tickets](tickets/README.md), so keep one deliverable per bullet. A clause ID is the phase key plus the bullet's position, such as `v0.3`. Run `python3 scripts/check-tickets.py --clauses` to list them.

## p0: Phase 0, spike (1 to 2 days)

Build nothing permanent.

- Confirm the four tree-sitter grammars compile with tree-sitter 0.27.
- Run fallow 3.42 on a fixture TS repo with `dead-code`, `dupes`, `health` as SARIF and the bare run as JSON. Record which findings and locations each format carries, and the wall time.
- Write a short note that fixes the SARIF versus JSON decision in [fallow behavior](reference/fallow.md#behavior).

## v0: smallest useful tool (about 1 to 2 weeks)

- `slopr check` on TS, TSX, JS, Python, Rust.
- The v0 rules in the [rule catalog](reference/rules.md#merged-rule-catalog).
- fallow passthrough.
- fallow startup version check and install-hint error ([decision record](research/decisions.md#decision-record-2026-11-11)).
- human output.
- json output.
- sarif output.
- Inline suppressions with reasons.
- Exit codes.
- `slopr init` writes `slopr.toml` with defaults.
- `slopr rules` lists the native rules.
- Fixture harness.
- CLI snapshots.
- fallow contract test.

Done when:

- the contract test passes against fallow 3.42.0.
- every v0 rule has bad and good fixtures.
- the FP corpus shows each default-on rule at or below 1 finding per 1,000 lines.

## v1: diffs and breadth

- `--changed-since` with line-level filtering.
- `--staged` with line-level filtering.
- Diff scans run `fallow audit`.
- Baseline save and compare.
- Go grammar.
- Rules marked v1 in the [rule catalog](reference/rules.md#merged-rule-catalog), including native complexity and exact duplication for non-JS languages and the two `pr/*` rules.
- `slopr rules --all` via `fallow schema`.
- `slopr explain`.
- `skill/SKILL.md`, printed by `slopr skill`.
- pre-commit hook.
- Release files: `LICENSE-MIT`, `LICENSE-APACHE`, and a complete `THIRD_PARTY_NOTICES.md`.

## v2: optional and model-based

- `dup/semantic` as an opt-in `--semantic` mode, clean-room from the slopo idea: tree-sitter units, embeddings from a configured OpenAI-compatible endpoint or local Ollama, cosine pairs, transitive clusters, distance boost, cluster hash ignore list. For JS/TS prefer `fallow similar-code`.
- Python `deps/undeclared-import`.
- Native `test/weakened`.
- Consider an MCP server (rmcp) only if agents need more than the JSON CLI.
- Consider a `cargo-fallow` adapter for Rust targets using the same subprocess pattern.

## Risks

- Comment heuristics produce false positives and annoy users. Mitigation: FP corpus gate, `comment/obvious` off by default, every finding suppressible with a reason.
- fallow moves fast (five releases in ten days). Mitigation: rely only on semver-covered surfaces, pin a range, weekly contract test against latest.
- fallow missing or out of range on the machine (users install it separately, per the 2026-11-11 decision). Mitigation: the startup version check in [fallow behavior](reference/fallow.md#behavior) step 3.
- Double reporting between native rules and fallow. Mitigation: `via_fallow_for_js` flag in the rule table.
- tree-sitter grammar crates need a C compiler and lag tree-sitter core releases. Mitigation: phase 0 build check; keep the grammar list short.
- License contamination. Mitigation: no code from slopo (AGPL), vibecheck (FSL), slop-guard (no license) or aicomplex (no LICENSE file); phrase lists only from MIT or Apache-2.0 sources with provenance comments in `phrases.rs`.
- Phrase lists are English-centric. stop-ai-slop has ru, de, fr, es lists under MIT that can be ported later.

## Open questions

Resolved: separate fallow install, see the [decision record](research/decisions.md#decision-record-2026-11-11).
