---
id: D-CONFIG
title: Decide rule settings, config loading, suppressions and baseline format
kind: decision
depends_on: []
docs: [docs/reference/config.md#config-sloprtoml, docs/reference/cli.md#cli]
touches: [docs/reference/config.md]
creates: [docs/reference/config.md#rule-settings, docs/reference/config.md#loading-and-precedence, docs/reference/config.md#inline-suppressions, docs/reference/config.md#baseline-file]
covers: [gap:D01, gap:D02, gap:D10, gap:D11]
signoff: user
verify: for h in "Rule settings" "Loading and precedence" "Inline suppressions" "Baseline file"; do grep -qx "## $h" docs/reference/config.md || exit 1; done
---

## Goal

Write the config, suppression and baseline decisions into `config.md`.

## Scope

Move the existing inline-suppression paragraph into `## Inline suppressions`, then add:

- `## Rule settings`. `RuleSettings` is one struct in `config.rs`, one typed field per row below, with `Default` giving the defaults. A field is added by the ticket of the first rule that reads it. `[rules.settings."<id>"]` may set only the keys listed for that rule. Any other rule ID or key exits 2. An invalid regex exits 2. `[[overrides]]` cannot change settings.

  | Rule | Key | Field | Type | Default |
  |---|---|---|---|---|
  | `comment/untracked-todo` | `issue_pattern` | `todo_issue_pattern` | regex | `(#\d+\|[A-Z]+-\d+\|https?://)` |
  | `comment/ai-vocab-density` | `min_distinct` | `vocab_min_distinct` | integer | 3 |
  | `complexity/function` | `max_cyclomatic`, `max_cognitive`, `max_lines`, `max_params`, `max_nesting` | `function_max_*` | integer | 10, 15, 80, 6, 5 |
  | `complexity/file-size` | `max_lines` | `file_max_lines` | integer | 400 |
  | `dup/exact-block` | `min_tokens`, `min_lines` | `dup_min_tokens`, `dup_min_lines` | integer | 50, 6 |
  | `pr/no-tests-changed` | `min_added_lines` | `pr_min_added_lines` | integer | 150 |
  | `pr/oversized-diff` | `max_added_lines`, `max_files` | `pr_max_added_lines`, `pr_max_files` | integer | 800, 15 |

- `## Loading and precedence`. slopr reads `slopr.toml` from the current directory, the scan root. With no file, defaults apply. PATHS default to `.`, and a PATH outside the root exits 2. Finding paths are root-relative with forward slashes. Severity values are `off`, `info`, `warn`, `error`. For rule R on path P, the severity is the rule's default (or `off` when `default_on` is false), then `[rules]`, then each matching `[[overrides]]` entry in file order, so the last match wins. Override and `exclude` globs are `globset` globs matched against root-relative paths. The walk uses `ignore::WalkBuilder` defaults plus `require_git(false)`, and `exclude` removes files after that. A PATH naming a file is scanned even when it is ignored, unless `exclude` matches it. The `[fallow]` table: `enabled` is `auto`, `on` or `off`. `bin` is a path, and `""` means unset. `version` is a `semver::VersionReq`. `commands` is a subset of `dead-code`, `dupes`, `health`, `security`, `architecture`, `flags`. `args` is a list of strings.
- `## Inline suppressions`. Directives are read line by line. In a parsed file, each line of each comment node counts, with comment markers and leading whitespace stripped. A block comment counts as several lines, and its line number is the comment's start line plus the line's offset. In a text file, each line counts after stripping leading whitespace and one leading `#`, `//`, `--`, `;` or `<!--`. A line is a directive candidate only when it then starts with `slopr-ignore-`. Text that mentions `slopr-ignore-` later in a line is not a directive. Grammar: `slopr-ignore-(next-line|file) <ids> -- <reason>`, where `<ids>` is one or more rule IDs separated by commas, and each ID matches `[a-z0-9-]+/[a-z0-9-]+`. `next-line` hides findings whose `start_line` is the first non-blank line after the end of the comment that holds the directive (for a text file, after the directive's line). `file` hides findings anywhere in the file, wherever the directive sits. Directives apply to native and `fallow/*` findings alike. A directive that is malformed, has an empty reason, or names an ID that is neither a native rule nor `fallow/`-prefixed produces `suppress/invalid-directive` at warn on the directive's line. A candidate that does not match the grammar, such as `slopr-ignore-next-lien`, is malformed. An invalid directive suppresses nothing, even when some of its IDs are valid. That finding cannot be suppressed or configured. Unused directives are not reported.
- `## Baseline file`. JSON with 2-space indentation and a trailing newline: `{"schema_version": 1, "findings": [{"fingerprint", "rule", "path"}]}`, sorted by path, rule, fingerprint, without duplicates. `slopr baseline save <FILE>` runs the full `check` scan (config, inline suppressions, fallow) and writes every finding, whatever its severity and the gate. It exits 0, or 2 or 3 on the usual errors. `--baseline <FILE>` hides findings whose fingerprint is in the file. A missing or invalid file exits 2. Stale entries are ignored.

## Out of scope

- Code.
- Diff-mode filtering. That belongs to `D-DIFF`.

## Acceptance criteria

- The four sections exist with the content above.
- The existing example and unknown-key paragraph stay.
- `python3 scripts/check-tickets.py` passes.
