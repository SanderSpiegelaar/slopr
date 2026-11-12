---
id: D-FALLOW
title: Decide fallow run details left open by the spike
kind: decision
depends_on: [S00, D-OUTPUT, D-TESTING]
docs: [docs/reference/fallow.md#behavior, docs/reference/fallow.md#phase-0-findings, docs/reference/config.md#config-sloprtoml, docs/reference/cli.md#output-formats]
touches: [docs/reference/fallow.md, docs/research/decisions.md]
creates: [docs/reference/fallow.md#run-details, docs/reference/fallow.md#fallow-schema]
covers: [gap:D15, gap:D23]
signoff: user
verify: grep -qx '## Run details' docs/reference/fallow.md && grep -qx '## fallow schema' docs/reference/fallow.md && ! grep -q '^[0-9]*\. .*--format sarif' docs/reference/fallow.md && grep -qx '## fallow output format (2026-11-12)' docs/research/decisions.md
---

## Goal

Pin how slopr runs fallow and maps its results, using the spike's findings.

## Scope

Add `## Run details` to `fallow.md`:

- fallow runs once per command, with the process cwd as the scan root. With PATHS other than `.`, fallow still runs on the root, and its findings are kept only when their path is under a PATH.
- `auto` treats fallow as required when `package.json` sits at the root or in any directory the walk visits. The walk honors ignore files, so `node_modules` is skipped.
- A result path is either relative to the root or absolute. Normalize both to root-relative paths with forward slashes, and drop results outside the root.
- A result with several locations becomes one `Finding` per location, with the same rule and message plus ` (location i of n)`.
- fallow findings are fingerprinted like native ones, from the flagged lines of the file on disk. If the file cannot be read, the message stands in for the snippet.
- `fallow.args` are appended after `--format json --quiet` on every command.
- No timeout.
- Commands run one after another on a thread spawned at startup, which the pipeline joins before suppressions.
- The field mapping from the JSON envelope, which the spike chose: which arrays become findings and the rule ID for each, the severity mapping, 0-based JSON columns converted to 1-based, and end positions where fallow gives them.

Rewrite fallow.md steps 4 to 7 for JSON. Step 4 runs either one bare `fallow --format json --quiet` or one run per command, chosen from the Phase 0 findings, and says how `fallow.commands` selects what is mapped. Step 5 runs `fallow audit --base <ref> --format json --quiet`. Step 6 points at the field mapping in Run details. Step 7 says slopr builds the fallow SARIF run from the mapped findings, per cli.md Output formats.

Append `## fallow output format (2026-11-12)` to `docs/research/decisions.md`: slopr reads fallow's JSON, not its SARIF. The Phase 0 SARIF drops the second file of a cycle, the link between clone instances, the cognitive score, every end position, and whether an audit finding is new. slopr's SARIF output builds the fallow run from the mapped findings instead of appending fallow's runs. This replaces the SARIF parts of the 2026-11-11 record.

Add `## fallow schema`: how `slopr rules --all` reads `fallow schema` output (taken from the spike), and the printed line per issue type, `fallow/<id> <description>`.

## Out of scope

- Code.

## Acceptance criteria

- Both sections exist and agree with `## Phase 0 findings`.
- `python3 scripts/check-tickets.py` passes.
