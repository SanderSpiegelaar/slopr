---
id: R-DEBUG
title: v0 debug leftover rule
kind: code
depends_on: [T01, D-RULES-V0]
docs: [docs/reference/rules.md#merged-rule-catalog, docs/reference/rules.md#v0-language-coverage, docs/reference/architecture.md#core-types, docs/reference/architecture.md#rule-kinds, docs/reference/testing.md#test-strategy, docs/reference/architecture.md#test-files]
touches: [src/rules/debug.rs, tests/fixtures/rules/debug/leftover/]
covers: [rule:debug/leftover]
verify: ls tests/fixtures/rules/debug/leftover/{bad,good}.{ts,py,rs} >/dev/null && cargo test --test rules
---

## Goal

slopr flags debug output and breakpoints left in library code.

## Scope

- `rules/debug.rs`: `debug/leftover` per v0 language coverage.
- Fixtures in TS, PY, RS. `good.ts` starts with `#!`. `good.py` has an `if __name__ == "__main__":` block. `good.rs` has `println!` and a `dbg!` inside `#[cfg(test)]`.

## Out of scope

- Go (V03).

## Acceptance criteria

- `cargo test --test rules` passes.
