# fallow integration

## Behavior

1. Enablement. `fallow.enabled = "auto"` (default) requires fallow when the scan root, or a workspace under it, has a `package.json`. `"on"` always requires it. `"off"` skips fallow and its startup check.
2. Locate binary in order: `--fallow-bin`, config `fallow.bin`, `<root>/node_modules/.bin/fallow`, `PATH`.
3. Startup version check, before any analysis: run `fallow --version`, parse with `semver`, require `fallow.version` (default `>=3.42, <4`). When fallow is required and is missing or out of range, exit 3 with an error naming the found version, the required range and an install hint (`npm install -D fallow@^3.42`, `cargo install fallow-cli --version ^3.42`). `--fallow off` is the escape hatch.
4. Full scan runs, from the project root: `fallow dead-code --format sarif --quiet`, `fallow dupes --format sarif --quiet`, `fallow health --format sarif --quiet`. Optional commands from config `fallow.commands`: `security`, `architecture`, `flags`.
5. Diff scan (`--changed-since <ref>`) runs `fallow audit --base <ref> --format json --quiet` instead, which includes the diff-risk rules (`weakened-tests`, `added-suppression`, `test-only-branch`).
6. Parse each SARIF log with `serde_json::Value`, read only spec-required fields: `runs[].results[].ruleId`, `level`, `message.text`, `locations[0].physicalLocation.artifactLocation.uri`, `region.startLine`, `startColumn`, `endLine`. Map level `error` to Error, `warning` to Warn, `note` and `none` to Info. Prefix rule IDs with `fallow/` unless already prefixed.
7. SARIF output from slopr appends fallow's runs verbatim as extra `runs[]` entries, so GitHub code scanning shows fallow's own rule metadata. Human and JSON outputs use the mapped findings.
8. Exit codes from fallow: 0 and 1 are both success for slopr (slopr applies its own gate from finding severities). 2 or a signal is a tool failure: slopr exits 3.
9. Config is not translated. fallow reads its own `.fallowrc.json`, `.fallowrc.jsonc` or `fallow.toml`. slopr passes `fallow.args` through unchanged.
10. `slopr rules --all` runs `fallow schema` and lists every fallow issue type next to native rules.

Phase 0 spike must confirm on a fixture: that each of the three commands emits SARIF with locations for every finding kind (duplication results may carry several locations), and the wall time of three commands versus one bare `fallow --format json` run. If SARIF drops findings that JSON has, switch step 4 to the combined JSON envelope (schema_version 15) and map its arrays.

## Phase 0 findings

Run on 2026-11-12 on macOS, Apple silicon.

### Versions

- fallow 3.42.0, installed with `npm install fallow@3.42.0`. `fallow --version` printed `fallow 3.42.0` and a signed `@fallow-cli/darwin-arm64` binary. `node_modules/.bin/fallow` is a Node launcher script, and every timing below includes it.
- Rust 1.96.1 and Apple clang 21.0.0 for the grammar build. Crate versions are in [dependencies](dependencies.md).

### Fixture

A scratch TS repo with a `package.json` (`"main": "src/index.ts"`) and no `node_modules`. It has two commits. The edits for the audit run are uncommitted, because a local commit hook rejects commits that add fallow findings. Every run reported the `node-modules-missing` workspace diagnostic.

Planted issues:

- Unused export. `unusedHelper` in `src/utils.ts`.
- Unlisted dependency. `import chunk from "lodash/chunk"` in `src/index.ts`, with no dependencies in `package.json`.
- Duplicated block. The same 18-line function body in `src/orders.ts` and `src/invoices.ts`.
- Function over the cognitive limit. `classify` in `src/complex.ts`, cognitive 59, cyclomatic 20.
- Circular import. `src/cycle-a.ts` and `src/cycle-b.ts` import each other.

The uncommitted edits add a second unused export (`addedUnused`) and `src/complex2.ts`, a copy of `classify` named `classifyAgain`. These also show up in the full-scan results.

### Findings per planted issue

Every planted issue appears in both formats. SARIF loses locations and detail on three of them.

| Issue | SARIF result | JSON (bare `fallow --format json`) |
|---|---|---|
| Unused export | `fallow/unused-export`, one location, `src/utils.ts` line 5, column 17. | `check.unused_exports[]`, `path`, `line` 5, `col` 16, `span_start`. |
| Unlisted dependency | `fallow/unlisted-dependency`, one location, `src/index.ts` line 7, column 8. | `check.unlisted_dependencies[]`, `package_name`, and `imported_from[]` with `path`, `line` and `col` for every import site. |
| Duplicated block | `fallow/code-duplication`, one result per instance, each with one location (start line and column only). No field links the two results except the message text, `Code clone group 1 (18 lines, 2 instances)`. | `dupes.clone_groups[].instances[]`, each with `file`, `start_line`, `end_line`, `start_col`, `end_col` and `fragment`. |
| Cognitive limit | `fallow/high-crap-score` only, `src/complex.ts` line 1, column 8. No `fallow/high-cognitive-complexity` result, although the run declares that rule. The cognitive value appears only in a `fallow/refactoring-target` message. | `health.findings[]` with `cyclomatic` 20, `cognitive` 59, `crap` 420.0 and `exceeded` `cognitive_crap`. |
| Circular import | `fallow/circular-dependency`, one location, `src/cycle-a.ts` line 1, column 10. The second file appears only in the message. | `check.circular_dependencies[]` with `files` and `edges[]`, one `path`, `line` and `col` per import in the cycle. |

Other observations:

- No SARIF result has `endLine` or `endColumn`. Step 6 reads `endLine`.
- SARIF `startColumn` is 1-based. JSON `col` and `start_col` are 0-based. Lines are 1-based in both.
- `health` emitted five `fallow/refactoring-target` warnings. Each has an `artifactLocation` and no `region`.
- Every SARIF result has the partial fingerprints `tools.fallow.fingerprint/v1` and `primaryLocationLineHash/v1`. Some also have `fallowFinding/v1`, which equals the JSON `finding_id`.
- Each SARIF log is version 2.1.0 with one run. The driver is `fallow` 3.42.0 with 55 rules for `dead-code`, 1 for `dupes` and 22 for `health`.
- `--quiet` keeps stdout to the report. When a gate fails, stderr still gets one line, such as `[X] Exit code 1: gate error-severity-findings (4 at or above error) failed.`
- Exit codes were 1 for `dead-code`, 0 for `dupes`, 1 for `health` and 0 for bare JSON. Bare JSON enforces no gate unless `--fail-on-issues` is passed, and says so on stderr.
- The bare JSON envelope has `kind` `combined`, `schema_version` 15 and `version` 3.42.0. Its top-level keys are `gate_outcomes`, `check`, `dupes`, `health`, `workspace_diagnostics`, `next_steps` and `_meta`. `_meta.telemetry` holds an `analysis_run_id`.

### Wall times

Seconds, measured with bash `time`, three runs each, warm `.fallow/` cache. The fixture has eight source files, so these numbers compare process overhead, not analysis cost on a real repo.

| Command | Run 1 | Run 2 | Run 3 |
|---|---|---|---|
| `fallow dead-code --format sarif --quiet` | 0.068 | 0.065 | 0.065 |
| `fallow dupes --format sarif --quiet` | 0.066 | 0.064 | 0.064 |
| `fallow health --format sarif --quiet` | 0.107 | 0.104 | 0.105 |
| The three above in sequence | 0.236 | 0.233 | 0.236 |
| `fallow --format json --quiet` | 0.151 | 0.146 | 0.144 |

One bare JSON run takes about 62 percent of the time of the three SARIF runs. The likely cause, not measured, is that each process pays launcher startup and rebuilds the module graph.

### `fallow audit --base HEAD~1 --format sarif --quiet`

The run exited 1 and counted four changed files, which included the uncommitted edits. The log has three runs, in this order:

1. Dead code, 55 rules. It reports every dead-code finding in a changed file, inherited or introduced, with no property to tell them apart. Here that was `unusedHelper`, `addedUnused` and `lodash`. The cycle was not reported, because neither of its files changed.
2. Duplication, with `automationDetails.id` `fallow/audit/dupes` and no rules. Its one result, `Clone group 1 (42 lines, 2 instances)`, has no `locations` at all.
3. Health, 22 rules. One `fallow/high-crap-score` result for `classifyAgain`, with `properties.attribution` `introduced`. The inherited `classify` was not reported.

With `--format json`, audit returns `kind` `audit` and `schema_version` 14. It has `attribution` counts, an `introduced` boolean on each dead-code finding, clone group, and complexity finding, and full line ranges on every clone instance.

### `fallow schema`

One JSON object on stdout, about 240 KB, exit 0. Top-level keys are `name`, `version`, `manifest_version` (`"1"`), `description`, `global_flags`, `commands`, `default_command`, `default_behavior`, `issue_types`, `suppression_comments`, `output_formats`, `exit_codes`, `environment_variables`, `severity_levels`, `field_notes`, `related_schemas`, `boundary_presets`, `taste_choices`, `security_categories`, `mcp_tools`, `mcp_resources`, `plugins` and `task_matrix`.

`issue_types` is an array of 134 objects. Each has `id`, `rule_id` (`fallow/<id>`), `command`, `category`, `description`, `default_severity`, `opt_in`, `sarif_rule_ids`, `result_key` (the JSON envelope array name, or null) and `docs_url`, among other fields. By `command` there are 55 `dead-code`, 48 `security`, 22 `health`, 5 `audit`, 2 `flags`, 1 `dupes` and 1 `mutation`. `severity_levels` is `error`, `warn`, `off`.

### Decision

Switch step 4 to one `fallow --format json --quiet` run and map the arrays of the combined envelope (`schema_version` 15). Switch step 5 to `fallow audit --base <ref> --format json --quiet` (`schema_version` 14). Reasons:

- SARIF drops locations that JSON has. It loses the second file of a cycle, every end line, and every location of an audit clone group.
- SARIF reports the cognitive-limit function only as a CRAP score.
- Audit SARIF does not mark dead-code findings as introduced or inherited.
- One JSON run was faster than the three SARIF runs on the fixture.

This changes steps 4 to 7 above, which still describe SARIF. Step 6 needs a JSON field mapping, with JSON columns 0-based. Step 7 can no longer append fallow's own SARIF runs. `D-FALLOW` owns those edits.
