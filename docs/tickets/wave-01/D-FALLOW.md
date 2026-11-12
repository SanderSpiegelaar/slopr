---
id: D-FALLOW
title: Decide fallow run details left open by the spike
kind: decision
depends_on: [S00]
docs: [docs/reference/fallow.md#behavior, docs/reference/fallow.md#phase-0-findings, docs/reference/config.md#config-sloprtoml]
touches: [docs/reference/fallow.md]
creates: [docs/reference/fallow.md#run-details, docs/reference/fallow.md#fallow-schema]
covers: [gap:D15, gap:D23]
signoff: user
verify: grep -qx '## Run details' docs/reference/fallow.md && grep -qx '## fallow schema' docs/reference/fallow.md
---

## Goal

Pin how slopr runs fallow and maps its results, using the spike's findings.

## Scope

Add `## Run details` to `fallow.md`:

- fallow runs once per command, with the process cwd as the scan root. With PATHS other than `.`, fallow still runs on the root, and its findings are kept only when their path is under a PATH.
- `auto` treats fallow as required when `package.json` sits at the root or in any directory the walk visits. The walk honors ignore files, so `node_modules` is skipped.
- A result URI is either relative to the root or a `file://` URI. Normalize both to root-relative paths with forward slashes, and drop results outside the root.
- A result with several locations becomes one `Finding` per location, with the same rule and message plus ` (location i of n)`.
- fallow findings are fingerprinted like native ones, from the flagged lines of the file on disk. If the file cannot be read, the message stands in for the snippet.
- `fallow.args` are appended after `--format sarif --quiet` on every command.
- No timeout.
- Commands run one after another on a thread spawned at startup, which the pipeline joins before suppressions.
- If the spike chose the JSON envelope, write the field mapping here in place of fallow.md step 6, and update steps 4 and 6 to point at it.

Add `## fallow schema`: how `slopr rules --all` reads `fallow schema` output (taken from the spike), and the printed line per issue type, `fallow/<id> <description>`.

## Out of scope

- Code.

## Acceptance criteria

- Both sections exist and agree with `## Phase 0 findings`.
- `python3 scripts/check-tickets.py` passes.
