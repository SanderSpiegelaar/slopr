# Test strategy

- Fixture harness (`tests/rules.rs`): for each `tests/fixtures/rules/<rule-id>/`, every line in `bad.*` that ends with a marker comment `expect: <rule-id>` must produce exactly that finding, and `good.*` must produce none. One harness, no per-rule test code.
- Seed `good.*` with the documented false-positive cases from the sources: aislop "Rule notes" (Python docstring URLs, `exc_info=True`, call-wrapped mutable defaults), abstract methods for `stub/placeholder-body`, CLI `main` files for `debug/leftover`, tool directives (`eslint-disable`, `noqa`) for comment rules (sloplint `src/languages.ts`).
- CLI snapshots (`tests/cli.rs`, insta + assert_cmd): human, json, sarif output on `fixtures/repos/*`, and each exit code path (0, 1, 2, 3).
- SARIF validity: validate output against the SARIF 2.1.0 JSON schema in a dev-only test.
- fallow contract test (`tests/fallow_contract.rs`, skipped unless `FALLOW_BIN` is set): CI installs `fallow@3.42.0` from npm, runs slopr on `fixtures/repos/ts-app` with planted issues (unused export, unlisted dependency, duplicated block, function over the cognitive ceiling, circular import) and asserts the matching `fallow/*` rule IDs appear. A weekly CI job runs the same test against the latest fallow to catch drift early.
- False-positive corpus: a script runs `slopr check --format json` over pinned commits of five mature repos (for example ripgrep, flask, zod, cobra, fallow itself) and prints findings per 1,000 lines per rule. A rule that exceeds 1 finding per 1,000 lines on mature code stays off by default. This mirrors aislop `docs/oss-benchmark.md` and stop-ai-slop `detbench-corpus.json`.
- Determinism: run twice, byte-compare JSON output.
