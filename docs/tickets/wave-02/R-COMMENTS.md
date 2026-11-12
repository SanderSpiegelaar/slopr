---
id: R-COMMENTS
title: v0 comment rules
kind: code
depends_on: [T01, D-RULES-V0, D-SOURCES]
docs: [docs/reference/rules.md#merged-rule-catalog, docs/reference/rules.md#v0-language-coverage, docs/reference/architecture.md#core-types, docs/reference/architecture.md#rule-kinds, docs/reference/testing.md#test-strategy, docs/research/sources.md#ported-lists, docs/reference/licensing.md#licensing, docs/reference/config.md#rule-settings]
touches: [src/rules/comments.rs, src/rules/phrases.rs, tests/fixtures/rules/comment/narrative/, tests/fixtures/rules/comment/section-divider/, tests/fixtures/rules/comment/step-numbered/, tests/fixtures/rules/comment/agent-artifact/, tests/fixtures/rules/comment/untracked-todo/]
covers: [rule:comment/narrative, rule:comment/section-divider, rule:comment/step-numbered, rule:comment/agent-artifact, rule:comment/untracked-todo]
verify: ls tests/fixtures/rules/comment/{narrative,section-divider,step-numbered,agent-artifact,untracked-todo}/{bad,good}.py >/dev/null && cargo test --test rules
---

## Goal

slopr flags the comment slop agents leave: narration, dividers, step headers, agent chatter and untracked TODOs.

## Scope

- `rules/phrases.rs` (new, see the note below): the narrative and agent-artifact lists copied from the pinned files in Ported lists. Each list has a comment naming repo, path, SHA and license. Use `aho-corasick` for the phrase scan.
- `rules/comments.rs`: the five rules per v0 language coverage, built on `SourceFile::comments()` and `strings()`. Severities from the catalog.
- Fixtures per rule: `bad.ts`, `bad.py`, `bad.rs`, `good.ts`, `good.py`, `good.rs`, with `expect:` markers. Seed `good.*` with tool directives (`eslint-disable`, `noqa`), doc comments, Python docstrings, and TODOs with `#123`, `ABC-12` or a URL.

Note: declare `phrases` as `#[path = "phrases.rs"] mod phrases;` inside `comments.rs`, so this ticket does not touch `rules/mod.rs`. V05 adds the v1 lists to the same file.

## Out of scope

- v1 comment rules (V05).
- Changing `rules/mod.rs`, `source.rs` or `Cargo.toml`. If a helper is missing there, report it instead.

## Acceptance criteria

- `cargo test --test rules` passes with all 30 fixture files.
- Every ported list carries its provenance comment.
