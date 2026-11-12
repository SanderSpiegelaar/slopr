---
id: R-TESTS
title: v0 test rule
kind: code
depends_on: [T01, D-RULES-V0]
docs: [docs/reference/rules.md#merged-rule-catalog, docs/reference/rules.md#v0-language-coverage, docs/reference/architecture.md#core-types, docs/reference/architecture.md#rule-kinds, docs/reference/testing.md#test-strategy, docs/reference/architecture.md#test-files]
touches: [src/rules/tests.rs, tests/fixtures/rules/test/tautological/]
covers: [rule:test/tautological]
verify: ls tests/fixtures/rules/test/tautological/{bad.test.ts,bad_test.py,bad.rs,good.test.ts,good_test.py,good.rs} >/dev/null && cargo test --test rules
---

## Goal

slopr flags tests that assert nothing.

## Scope

- `rules/tests.rs`: `test/tautological` per v0 language coverage, using `SourceFile::in_test`.
- Fixtures named so they count as tests: `bad.test.ts`, `bad_test.py`, `bad.rs` (assertions inside `#[cfg(test)] mod tests`), and good counterparts. `good.rs` has `assert!(true)` outside test code, which must not fire.

## Out of scope

- `test/weakened` (E03).

## Acceptance criteria

- `cargo test --test rules` passes.
