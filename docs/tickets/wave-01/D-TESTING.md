---
id: D-TESTING
title: Decide SARIF checks, the FP corpus and CI
kind: decision
depends_on: [D-ARCH]
docs: [docs/reference/testing.md#test-strategy, docs/reference/dependencies.md#dependencies-versions-from-cratesio-on-2026-11-11]
touches: [docs/reference/testing.md, docs/research/decisions.md]
creates: [docs/reference/testing.md#sarif-checks, docs/reference/testing.md#fp-corpus, docs/reference/testing.md#ci]
covers: [gap:D14, gap:D21, gap:D22]
signoff: user
verify: for h in "SARIF checks" "FP corpus" "CI"; do grep -qx "## $h" docs/reference/testing.md || exit 1; done; ! grep -q 'JSON schema in a dev-only test' docs/reference/testing.md
---

## Goal

Replace the test items that would need an unapproved crate or an undefined setup with concrete ones.

## Scope

In `testing.md`, replace the SARIF validity bullet with a pointer to the new section, then add:

- `## SARIF checks`. No schema validator crate. `tests/cli.rs` asserts on parsed SARIF output: `version == "2.1.0"`, `$schema` set, `runs[0].tool.driver.name == "slopr"`, a rule entry for every `ruleId` used, and each result having `ruleId`, `level`, `message.text`, `locations[0].physicalLocation.region.startLine`, and `partialFingerprints["slopr/v1"]`.
- `## FP corpus`. `scripts/fp_corpus.py` (Python 3 stdlib only) clones these into `target/fp-corpus/` with `git clone --depth 1 --branch <tag>`, reusing an existing clone: `BurntSushi/ripgrep` at `14.1.1`, `pallets/flask` at `3.1.0`, `colinhacks/zod` at `v3.24.1`, `spf13/cobra` at `v1.8.1`, `fallow-rs/fallow` at `v3.42.0`. If a tag is missing, use the nearest older tag and update this list. It runs `slopr check --format json --fallow off` in each clone and counts non-blank lines of files whose extension maps to a parsed language. A rule's rate is its findings per 1,000 lines over the corpus files in its languages. It prints a rule by repo table. With `--check`, it exits 1 when a `default_on` rule has a rate above 1.0. It reads `default_on` from `slopr rules` output. Go files count only once Go is supported. The corpus is not in CI, because it needs the network.
- `## CI`. `.github/workflows/ci.yml` on push and pull_request: a `test` job on `ubuntu-latest` with stable Rust running `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`. A `fallow-contract` job installs Node 22 and `npm install -g fallow@3.42.0`, then runs `FALLOW_BIN=$(command -v fallow) cargo test --test fallow_contract`. A `schedule` trigger (cron `0 6 * * 1`) runs the contract job with `fallow@latest`.

Append `## SARIF validation (2026-11-12)` to `decisions.md`: the schema test is replaced by field assertions, because a validator crate is not in the approved list.

## Out of scope

- Writing the script or the workflow. `G0`, `C01` and `F03` do that.

## Acceptance criteria

- The three sections exist with the content above, and the old schema bullet is gone.
- `python3 scripts/check-tickets.py` passes.
