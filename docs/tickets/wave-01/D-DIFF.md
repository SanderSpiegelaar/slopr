---
id: D-DIFF
title: Decide diff-mode semantics
kind: decision
depends_on: [D-OUTPUT]
docs: [docs/reference/cli.md#cli, docs/reference/fallow.md#behavior, docs/reference/rules.md#merged-rule-catalog]
touches: [docs/reference/cli.md]
creates: [docs/reference/cli.md#diff-mode]
covers: [gap:D18]
signoff: user
verify: grep -qx '## Diff mode' docs/reference/cli.md
---

## Goal

Pin what "changed lines" means, so `--changed-since`, `--staged` and the `pr/*` rules have one definition.

## Scope

Add `## Diff mode` to `cli.md`:

- `--changed-since <REF>` diffs the working tree against `git merge-base <REF> HEAD`, via `git diff --unified=0 --no-color --no-ext-diff -M <base>`. Untracked files that are not ignored (`git ls-files --others --exclude-standard`) count as fully changed.
- `--staged` uses `git diff --cached --unified=0 --no-color --no-ext-diff -M`. Rules read the working-tree content of each file, because the pre-commit framework stashes unstaged changes first.
- Passing both flags, or either one outside a git work tree, exits 2.
- Changed lines are the new-side line ranges of each hunk. Deletion-only hunks add none. A renamed file uses its new path.
- Only changed files are scanned. A finding is kept when its span intersects a changed line. A file-level finding, one whose span covers the whole file, is kept when the file has any changed line.
- fallow in diff mode runs `fallow audit --base <base> --format json --quiet` in place of the full-scan commands, with `<base>` being the merge base, or `HEAD` for `--staged`. Its findings are not line-filtered.
- `pr/*` findings use the changed file with the most added lines (ties broken by path order), with a span on that file's first changed line. `pr/no-tests-changed` fires when the added lines in parsed non-test files reach `pr_min_added_lines` and no changed file is a test file. `pr/oversized-diff` fires when added lines exceed `pr_max_added_lines` or changed files exceed `pr_max_files`. Both run only in diff mode.

## Out of scope

- Code.

## Acceptance criteria

- The section exists with the content above.
- `python3 scripts/check-tickets.py` passes.
