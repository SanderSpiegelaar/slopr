# Test strategy

- Fixture harness (`tests/rules.rs`): for each `tests/fixtures/rules/<rule-id>/`, every line in `bad.*` that ends with a marker comment `expect: <rule-id>` must produce exactly that finding, and `good.*` must produce none. One harness, no per-rule test code.
- Seed `good.*` with the documented false-positive cases from the sources: aislop "Rule notes" (Python docstring URLs, `exc_info=True`, call-wrapped mutable defaults), abstract methods for `stub/placeholder-body`, CLI `main` files for `debug/leftover`, tool directives (`eslint-disable`, `noqa`) for comment rules (sloplint `src/languages.ts`).
- CLI snapshots (`tests/cli.rs`, insta + assert_cmd): human, json, sarif output on `fixtures/repos/*`, and each exit code path (0, 1, 2, 3).
- SARIF validity: see [SARIF checks](#sarif-checks).
- fallow contract test (`tests/fallow_contract.rs`, skipped unless `FALLOW_BIN` is set): CI installs `fallow@3.42.0` from npm, runs slopr on `fixtures/repos/ts-app` with planted issues (unused export, unlisted dependency, duplicated block, function over the cognitive ceiling, circular import) and asserts the matching `fallow/*` rule IDs appear. A weekly CI job runs the same test against the latest fallow to catch drift early.
- False-positive corpus: a script runs `slopr check --format json` over pinned commits of five mature repos (for example ripgrep, flask, zod, cobra, fallow itself) and prints findings per 1,000 lines per rule. A rule that exceeds 1 finding per 1,000 lines on mature code stays off by default. This mirrors aislop `docs/oss-benchmark.md` and stop-ai-slop `detbench-corpus.json`.
- Determinism: run twice, byte-compare JSON output.

## SARIF checks

slopr uses no SARIF schema validator crate. `tests/cli.rs` parses the SARIF output and asserts on its fields:

- `version == "2.1.0"`, and `$schema` is set.
- `runs[0].tool.driver.name == "slopr"`.
- `runs[0].tool.driver.rules` has an entry for every `ruleId` used in the results.
- Each result has `ruleId`, `level`, `message.text`, `locations[0].physicalLocation.region.startLine` and `partialFingerprints["slopr/v1"]`.

## FP corpus

`scripts/fp_corpus.py` uses only the Python 3 standard library. It clones each repo into `target/fp-corpus/` with `git clone --depth 1 --branch <tag>`, and reuses an existing clone:

| Repo | Tag |
|---|---|
| `BurntSushi/ripgrep` | `14.1.1` |
| `pallets/flask` | `3.1.0` |
| `colinhacks/zod` | `v3.24.1` |
| `spf13/cobra` | `v1.8.1` |
| `fallow-rs/fallow` | `v3.42.0` |

If a tag is missing, use the nearest older tag and update this table.

In each clone the script runs `slopr check --format json --fallow off`. It counts non-blank lines in files whose extension maps to a parsed language. Go files count only once slopr supports Go. A rule's rate is its findings per 1,000 lines over the corpus files in that rule's languages. The script prints a table of rules by repo.

With `--check`, the script exits 1 when a rule with `default_on` set has a rate above 1.0. It reads `default_on` from the `slopr rules` output.

The corpus does not run in CI, because it needs the network.

## CI

`.github/workflows/ci.yml` runs on `push` and `pull_request`, with two jobs:

- `test` on `ubuntu-latest` with stable Rust runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`.
- `fallow-contract` installs Node 22 and runs `npm install -g fallow@3.42.0`, then `FALLOW_BIN=$(command -v fallow) cargo test --test fallow_contract`.

A `schedule` trigger (cron `0 6 * * 1`) runs the `fallow-contract` job with `fallow@latest` instead.
