---
id: D-SOURCES
title: Pin the upstream source of every ported list and write the notices
kind: decision
depends_on: []
docs: [docs/reference/licensing.md#licensing, docs/research/sources.md#per-repo-notes-code-read-paths-cited, docs/reference/rules.md#merged-rule-catalog]
touches: [docs/research/sources.md, THIRD_PARTY_NOTICES.md]
creates: [docs/research/sources.md#ported-lists]
covers: [gap:D09]
verify: grep -qx '## Ported lists' docs/research/sources.md && grep -q 'heavykenny' THIRD_PARTY_NOTICES.md && grep -q 'WhiteBite' THIRD_PARTY_NOTICES.md
---

## Goal

Give every rule that ports a list one pinned upstream file, so rule tickets copy from a fixed commit with the right notice.

## Scope

- Shallow-clone `scanaislop/aislop` and `WhiteBite/stop-ai-slop` from GitHub. Network use is fine here, because only `slopr check` must stay offline.
- Add `## Ported lists` to `sources.md`, a table with the columns: list name, slopr rule, repo, file path, commit SHA (full), license. One row for each of:
  - narrative openers (`comment/narrative`): aislop `narrative-comments-patterns.ts` and stop-ai-slop's `vend/this-function-opener` source file.
  - agent artifacts (`comment/agent-artifact`): aislop's `ai-slop/meta-comment` patterns and stop-ai-slop's `vend/ai-plan-narration` list.
  - changelog markers (`comment/changelog`): stop-ai-slop's `changelog-marker` list and aislop's `CROSS_REFERENCE_PHRASES`.
  - AI vocabulary (`comment/ai-vocab-density`): stop-ai-slop's `vend/ai-vocab-density` word list.
  - invisible code points (`text/invisible-unicode`): stop-ai-slop's `vend/zero-width-chars` and `vend/bidi-controls`, as provenance for the ranges in the catalog.
- Add a second table of lists that slopr writes itself, because their sources may not be copied: secret patterns (`sec/hardcoded-secret`), section dividers, step headers, vague error messages (`error/vague-message`), generic names (`naming/generic`).
- Create `THIRD_PARTY_NOTICES.md` at the repo root, with one section per repo in the first table holding its full MIT license text and copyright line (`Copyright 2025 heavykenny`, `Copyright 2026 WhiteBite`).
- If a named list does not exist at the pinned commit, record the closest file that holds it, and say so in the row.

## Out of scope

- Copying the lists into Rust. Rule tickets do that.

## Acceptance criteria

- Every row has a full 40-char SHA and a file path that exists at that SHA.
- `THIRD_PARTY_NOTICES.md` holds both MIT texts.
- `python3 scripts/check-tickets.py` passes.
