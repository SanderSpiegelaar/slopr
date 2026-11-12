---
id: R-TEXT
title: v0 text rules, invisible Unicode and hardcoded secrets
kind: code
depends_on: [T01, D-RULES-V0, D-SOURCES]
docs: [docs/reference/rules.md#merged-rule-catalog, docs/reference/rules.md#v0-language-coverage, docs/reference/architecture.md#core-types, docs/reference/architecture.md#rule-kinds, docs/reference/testing.md#test-strategy, docs/research/sources.md#ported-lists]
touches: [src/rules/text.rs, tests/fixtures/rules/text/invisible-unicode/, tests/fixtures/rules/sec/hardcoded-secret/]
covers: [rule:text/invisible-unicode, rule:sec/hardcoded-secret]
verify: ls tests/fixtures/rules/text/invisible-unicode/{bad.ts,bad.md,good.ts} tests/fixtures/rules/sec/hardcoded-secret/{bad.py,bad.env,good.py} >/dev/null && cargo test --test rules
---

## Goal

slopr catches hidden characters and leaked credentials in any file.

## Scope

- `rules/text.rs`: both rules with `langs: &[]`, scanning `SourceFile::text`. The code point list cites stop-ai-slop per Ported lists. The secret regexes are slopr's own.
- Fixtures: invisible-unicode `bad.ts`, `bad.md` (text file), `good.ts` (starting with a BOM). hardcoded-secret `bad.py`, `bad.env`, `good.py` (short `sk-` strings, `AKIA` in prose). Use clearly fake tokens such as AWS's documented `AKIAIOSFODNN7EXAMPLE`. If GitHub push protection blocks a token, swap it for another fake that still matches.
- A test in `text.rs` that the message never contains the full match.

## Out of scope

- Entropy-based detection.

## Acceptance criteria

- `cargo test --test rules` and `cargo test --bin slopr` pass.
- A BOM at offset 0 gives no finding. A BOM elsewhere does.
