---
id: R-ERRORS
title: v0 error-handling rules
kind: code
depends_on: [T01, D-RULES-V0]
docs: [docs/reference/rules.md#merged-rule-catalog, docs/reference/rules.md#v0-language-coverage, docs/reference/architecture.md#core-types, docs/reference/architecture.md#rule-kinds, docs/reference/testing.md#test-strategy]
touches: [src/rules/errors.rs, tests/fixtures/rules/error/swallowed/, tests/fixtures/rules/error/redundant-rethrow/]
covers: [rule:error/swallowed, rule:error/redundant-rethrow]
verify: ls tests/fixtures/rules/error/swallowed/{bad,good}.{ts,py,rs} tests/fixtures/rules/error/redundant-rethrow/{bad,good}.{ts,py} >/dev/null && cargo test --test rules
---

## Goal

slopr flags error handling that hides failures or adds nothing.

## Scope

- `rules/errors.rs`: `error/swallowed` and `error/redundant-rethrow` per v0 language coverage.
- Fixtures: swallowed in TS, PY and RS. redundant-rethrow in TS and PY. `good.py` holds `logger.exception(...)`, `exc_info=True`, and a handler that logs the error then re-raises. `good.ts` holds a catch that wraps and rethrows a new error.

## Out of scope

- v1 error rules (V06).

## Acceptance criteria

- `cargo test --test rules` passes.
