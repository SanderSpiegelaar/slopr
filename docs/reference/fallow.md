# fallow integration

## Behavior

1. Enablement. `fallow.enabled = "auto"` (default) requires fallow when the walk finds a `package.json`, see [Run details](#run-details). `"on"` always requires it. `"off"` skips fallow and its startup check.
2. Locate binary in order: `--fallow-bin`, config `fallow.bin`, `<root>/node_modules/.bin/fallow`, `PATH`.
3. Startup version check, before any analysis: run `fallow --version`, parse with `semver`, require `fallow.version` (default `>=3.42, <4`). When fallow is required and is missing or out of range, exit 3 with an error naming the found version, the required range and an install hint (`npm install -D fallow@^3.42`, `cargo install fallow-cli --version ^3.42`). `--fallow off` is the escape hatch.
4. Full scan. `<L>` is the comma-joined subset of `fallow.commands` among `dead-code`, `dupes`, `health` and `architecture`. When `<L>` is not empty, slopr runs one bare `fallow --only <L> --format json --quiet` from the scan root. It returns the combined envelope (`kind` `combined`, `schema_version` 15) with one `check`, `dupes` or `health` section per selected analysis. `architecture` selects the cycle, boundary and policy arrays of `check`. The bare envelope never holds security or flag findings, so `security` adds one `fallow security --format json --quiet` run (`kind` `security`, `schema_version` 8) and `flags` adds one `fallow flags --format json --quiet` run (`kind` `feature-flags`, `schema_version` 8).
5. Diff scan (`--changed-since <ref>`) runs `fallow audit --base <ref> --only <L> --format json --quiet` instead (`kind` `audit`, `schema_version` 14). The audit always adds its diff-risk findings (`weakened-tests`, `added-suppression`, `test-only-branch`). `security` and `flags` do not run in diff mode.
6. Parse the envelope with `serde_json::Value` and map it per the field mapping in [Run details](#run-details).
7. Human and JSON outputs use the mapped findings. SARIF output builds the fallow run from the mapped findings, per [Output formats](cli.md#output-formats). fallow's own SARIF is never read.
8. Exit codes from fallow: 0 and 1 are both success for slopr (slopr applies its own gate from finding severities). 2 or a signal is a tool failure: slopr exits 3.
9. Config is not translated. fallow reads its own `.fallowrc.json`, `.fallowrc.jsonc` or `fallow.toml`. slopr passes `fallow.args` through unchanged.
10. `slopr rules --all` runs `fallow schema` and lists every fallow issue type next to native rules, see [fallow schema](#fallow-schema).

The Phase 0 spike chose fallow's JSON over its SARIF, see [Phase 0 findings](#phase-0-findings).

## Run details

Each slopr command runs the fallow processes of step 4 or step 5 once, with the process cwd as the scan root. With PATHS other than `.`, fallow still runs on the root. Its findings are kept only when their path is under a PATH.

`auto` treats fallow as required when `package.json` sits at the root or in any directory the walk visits. The walk honors ignore files, so `node_modules` is skipped.

A result path is either relative to the root or absolute. Both are normalized to root-relative paths with forward slashes. Results outside the root are dropped.

A result with several locations becomes one `Finding` per location. Each has the same rule and message, plus ` (location i of n)` at the end of the message.

fallow findings are fingerprinted like native ones, from the flagged lines of the file on disk. If the file cannot be read, the message stands in for the snippet.

`fallow.args` are appended after `--format json --quiet` on every command.

There is no timeout.

Commands run one after another on a thread spawned at startup. The pipeline joins it before suppressions.

### Field mapping

Observed with fallow 3.42.0 on a scratch TS repo on 2026-11-12. These arrays become findings. Audit arrays sit in the audit envelope. Security and flag arrays sit at the top level of their own run.

| Full scan | Audit | Rule | Message | Severity |
|---|---|---|---|---|
| `check.<key>[]` | `dead_code.<key>[]` | `fallow/<id>` of the issue type whose `result_key` is `<key>` | `<description>: <subject>` | `effective_severity` |
| `dupes.clone_groups[]` | `duplication.clone_groups[]` | `fallow/code-duplication` | `<description>: <line_count> lines, <n> instances` | Warn |
| `health.findings[]` | `complexity.findings[]` | From `exceeded`, see below | `<description>: <name>, cyclomatic <cyclomatic>, cognitive <cognitive>, CRAP <crap>` | `effective_severity` |
| `security_findings[]` | Not run | `fallow/<category>` | `evidence` | Info |
| `feature_flags[]` | Not run | `fallow/feature-flag` | `<description>: <flag_name>` | Info |
| None | `diff_risk.findings[]` | `fallow/<rule>` | `message` | `severity` |

`<key>` is the bare snake_case array name, such as `unused_exports`. The same names appear in `check`, at the top level of a `dead-code` or `architecture` run, and in the audit's `dead_code`. In `fallow schema`, the 54 `dead-code` issue types have a `result_key`, and every other issue type has `null`. slopr embeds a table from `result_key` to `id` and `description`, plus the `description` of every other rule it builds a message for, taken from `fallow schema` 3.42.0. An array whose name is not in the table is ignored.

`<description>` is the issue type's `description`, such as `Export is never imported`. `<subject>` is the first field present among `export_name`, `member_name` (printed as `<parent_name>.<member_name>`), `package_name`, `specifier`, `import_specifier` and `local_name`. Without one, `files` joined with `, ` is the subject. Without that, the message is the description alone. A long file's message is `<description>: <line_count> lines`.

`exceeded` picks the health rule. These match the SARIF rule fallow reports for the same function:

| `exceeded` | Rule |
|---|---|
| `cyclomatic` | `fallow/high-cyclomatic-complexity` |
| `cognitive` | `fallow/high-cognitive-complexity` |
| `both` | `fallow/high-complexity` |
| `crap`, `cyclomatic_crap`, `cognitive_crap`, `all` | `fallow/high-crap-score` |
| `file_lines` | `fallow/long-file` |

A health finding's `severity` (`critical`, `high`, `moderate`) ranks findings and is not mapped. `health.targets[]`, which fallow's SARIF reports as `fallow/refactoring-target`, has no line and is not mapped.

### Locations

Each entry yields one location per element of its `edges[]`, `imported_from[]`, `locations[]` or `instances[]` array. An element without a `path` uses the entry's `path`. An entry without such an array yields one location from `path` (or `from_path`), `line` and `col`. Clone instances use `file`, `start_line`, `start_col`, `end_line` and `end_col`. Security `trace[]` is not a location list.

An entry with a `path` and no `line`, such as `unused_files`, and a health finding with `exceeded` `file_lines` are file-level findings whose span covers the whole file. An entry with no path is dropped. A location without `col`, such as `unused_dependencies`, starts at column 1. When fallow gives no end position, the end line and column equal the start.

Lines are 1-based and are used as they are. Every JSON column (`col`, `start_col`, `end_col`) is a 0-based UTF-8 byte offset into its line. On the line `/* ééé */ export const uni = 1;`, `col` for `uni` is 26, its byte offset, while its character offset is 23. slopr converts the byte offset to a count of Unicode scalar values through the file's line index, then adds 1. If the file cannot be read, it adds 1 to the byte offset.

`end_col` is exclusive. On the 32-byte line `  return total + tax + shipping;`, a clone instance has `end_col` 31, which stops before the `;`, and its `fragment` ends with `shipping`. slopr end columns are exclusive too, so `end_col` takes the same conversion.

### Severity

`effective_severity` is fallow's severity after its own config. A `.fallowrc.json` with `"rules": {"unused-export": "warn"}` changed `effective_severity` of the unused export to `warn`. `"off"` removed the entry. Setting `complexity-cyclomatic`, `complexity-cognitive` and `complexity-crap` to `warn` changed every health finding to `warn`. `high-crap-score` there had no effect.

`error` maps to Error and `warn` to Warn. Clone groups carry no severity, and fallow's SARIF reports them as `warning`. Security and flag findings carry no gated severity, and fallow's SARIF reports them as `note`. A security finding's `severity` (`low` in the run) ranks candidates and is not mapped.

### Audit

Every audit finding has an `introduced` boolean. Diff mode keeps only findings with `introduced: true`. In the run, the inherited `unusedHelper` in a changed `src/utils.ts` had `introduced: false`, and the new `addedUnused` beside it had `introduced: true`. `diff_risk` is absent when there are no diff-risk findings.

### Not mapped

`gate_outcomes`, `workspace_diagnostics`, `next_steps`, `_meta`, `dupes.clone_families` and `dupes.stats` are not mapped. Neither are the health `summary`, `sections`, `vital_signs`, `health_score`, `file_scores`, `hotspot_summary`, `change_coupling`, `targets` and `target_thresholds`, the security `config`, `unresolved_edge_files` and `unresolved_callee_sites`, the flags `total_flags`, or the audit `verdict`, `summary` and `attribution`.

## fallow schema

`slopr rules --all` runs `fallow schema` and parses the one JSON object it prints on stdout. It reads only `issue_types[]`, and from each entry only `id` and `description`. It prints one line per issue type, in array order, as `fallow/<id> <description>`. Security categories such as `command-injection` are issue types, so every `fallow/<category>` rule from [Field mapping](#field-mapping) is listed.

`result_key` holds the bare JSON array name, such as `unused_exports`, not a dotted path. Only `dead-code` issue types have one. `fallow schema` exited 0 in a directory with no `package.json`.

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
