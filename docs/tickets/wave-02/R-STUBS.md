---
id: R-STUBS
title: v0 stub rule
kind: code
depends_on: [T01, D-RULES-V0]
docs: [docs/reference/rules.md#merged-rule-catalog, docs/reference/rules.md#v0-language-coverage, docs/reference/architecture.md#core-types, docs/reference/architecture.md#rule-kinds, docs/reference/testing.md#test-strategy]
touches: [src/rules/stubs.rs, tests/fixtures/rules/stub/placeholder-body/]
covers: [rule:stub/placeholder-body]
verify: ls tests/fixtures/rules/stub/placeholder-body/{bad,good}.{ts,py,rs} >/dev/null && cargo test --test rules
---

## Goal

slopr flags functions an agent left as placeholders.

## Scope

- `rules/stubs.rs`: `stub/placeholder-body` per v0 language coverage.
- Fixtures in TS, PY, RS. `good.py` holds `@abstractmethod`, `@overload` and a `Protocol` class. `good.rs` holds a trait with a default `todo!()` method.

## Out of scope

- Go (V03).

## Acceptance criteria

- `cargo test --test rules` passes.
