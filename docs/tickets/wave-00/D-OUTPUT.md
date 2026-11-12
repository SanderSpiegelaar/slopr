---
id: D-OUTPUT
title: Decide the exit gate and output contracts
kind: decision
depends_on: []
docs: [docs/reference/cli.md#cli, docs/reference/cli.md#output-formats, docs/reference/architecture.md#core-types]
touches: [docs/reference/cli.md]
creates: [docs/reference/cli.md#exit-gate, docs/reference/cli.md#output-formats]
covers: [gap:D12, gap:D13]
verify: grep -qx '## Exit gate' docs/reference/cli.md && grep -qx '## Output formats' docs/reference/cli.md && ! grep -qx '## Output' docs/reference/cli.md
---

## Goal

Pin exit codes and every output byte, so CLI snapshots and the fixture harness have a fixed target.

## Scope

- Add `## Exit gate`. `--max-severity-exit` takes `info`, `warn` or `error` (default `error`). The gate fails, exit 1, when any reported finding (after suppressions, baseline and diff filtering) is at or above the level. Exit 2 (usage or config) happens before any scan, and clap usage errors already exit 2. Exit 3 (fallow required but missing, out of range, or failed) still prints the findings, then exits 3 whatever the gate says. A file that cannot be read prints `slopr: skipped <path>: <reason>` to stderr and does not change the exit code.
- Rename `## Output` to `## Output formats` and replace its bullets with:
  - Order: path (byte order), start_line, start_col, rule, message.
  - human: on stdout, one line per finding, `<path>:<line>:<col> <severity> <rule> <message>`, single spaces, with a blank line between files. Then a blank line, one `<count> <rule>` line per rule in rule order, and a last line `<n> findings (<e> error, <w> warn, <i> info)`. With no findings the whole output is `0 findings`. No color.
  - json: `{"schema_version": 1, "version": "<crate version>", "findings": [...], "summary": {"total": n, "by_severity": {"error": e, "warn": w, "info": i}, "by_rule": {"<rule>": n}}, "tools": [{"name": "fallow", "version": "<x.y.z>" or null, "status": "ok" | "skipped" | "missing" | "failed"}]}`. Each finding is the `Finding` struct with `span` as an object and `origin` as `{"kind": "native"}` or `{"kind": "fallow", "version", "command"}`. `by_rule` is sorted by key. Pretty-printed with 2 spaces and a trailing newline. The `fallow` entry is always present, with `skipped` when fallow is off or not required.
  - sarif: `{"$schema": "https://json.schemastore.org/sarif-2.1.0.json", "version": "2.1.0", "runs": [...]}`. The first run is slopr's. `tool.driver` has `name: "slopr"`, `version`, and `rules`, which lists every native rule plus `suppress/invalid-directive`, sorted by id. Each rule has `id`, `shortDescription.text` (its summary) and `defaultConfiguration.level`. Each result has `ruleId`, `level` (error to `error`, warn to `warning`, info to `note`), `message.text`, one location with `artifactLocation.uri` (the path) and `uriBaseId: "%SRCROOT%"`, a `region` with all four span fields, and `partialFingerprints: {"slopr/v1": fingerprint}`. fallow findings appear only in fallow's appended runs, never in slopr's run. No absolute paths anywhere, so snapshots stay stable.

## Out of scope

- Diff mode. That belongs to `D-DIFF`.

## Acceptance criteria

- Both sections exist with the content above. `## Output` is gone.
- `python3 scripts/check-tickets.py` passes.
