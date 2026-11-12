# Config: `slopr.toml`

```toml
exclude = ["dist/**", "**/*.min.js", "vendor/**"]

[rules]
"comment/untracked-todo" = "off"
"debug/leftover" = "error"

[rules.settings."comment/untracked-todo"]
issue_pattern = '(#\d+|[A-Z]+-\d+|https?://)'

[[overrides]]
paths = ["tests/**", "**/*.test.ts"]
rules = { "debug/leftover" = "off" }

[fallow]
enabled = "auto"
bin = ""
version = ">=3.42, <4"
commands = ["dead-code", "dupes", "health"]
args = []
```

Unknown keys are an error (exit 2), like fallow and slopo.

## Rule settings

`RuleSettings` is one struct in `config.rs`. It has one typed field per row below, and its `Default` gives the defaults. A field is added by the ticket of the first rule that reads it.

`[rules.settings."<id>"]` may set only the keys listed for that rule. Any other rule ID or key exits 2. An invalid regex exits 2. `[[overrides]]` cannot change settings.

| Rule | Key | Field | Type | Default |
|---|---|---|---|---|
| `comment/untracked-todo` | `issue_pattern` | `todo_issue_pattern` | regex | `(#\d+\|[A-Z]+-\d+\|https?://)` |
| `comment/ai-vocab-density` | `min_distinct` | `vocab_min_distinct` | integer | 3 |
| `complexity/function` | `max_cyclomatic`, `max_cognitive`, `max_lines`, `max_params`, `max_nesting` | `function_max_*` | integer | 10, 15, 80, 6, 5 |
| `complexity/file-size` | `max_lines` | `file_max_lines` | integer | 400 |
| `dup/exact-block` | `min_tokens`, `min_lines` | `dup_min_tokens`, `dup_min_lines` | integer | 50, 6 |
| `pr/no-tests-changed` | `min_added_lines` | `pr_min_added_lines` | integer | 150 |
| `pr/oversized-diff` | `max_added_lines`, `max_files` | `pr_max_added_lines`, `pr_max_files` | integer | 800, 15 |

## Loading and precedence

slopr reads `slopr.toml` from the current directory, which is the scan root. With no file, defaults apply. PATHS default to `.`, and a PATH outside the root exits 2. Finding paths are root-relative with forward slashes.

Severity values are `off`, `info`, `warn` and `error`. For rule R on path P, the severity starts at the rule's default, or `off` when `default_on` is false. `[rules]` replaces it. Then each matching `[[overrides]]` entry applies in file order, so the last match wins.

Override and `exclude` globs are `globset` globs matched against root-relative paths. The walk uses `ignore::WalkBuilder` defaults plus `require_git(false)`, and `exclude` removes files after that. A PATH naming a file is scanned even when it is ignored, unless `exclude` matches it.

The `[fallow]` table:

- `enabled` is `auto`, `on` or `off`.
- `bin` is a path, and `""` means unset.
- `version` is a `semver::VersionReq`.
- `commands` is a subset of `dead-code`, `dupes`, `health`, `security`, `architecture`, `flags`.
- `args` is a list of strings.

## Inline suppressions

Inline suppression: `slopr-ignore-next-line <rule-id> -- <reason>` and `slopr-ignore-file <rule-id> -- <reason>`. Rule ID and reason are required. A directive without them is itself a finding `suppress/invalid-directive` (aislop `ai-slop/unknown-directive` shows the need).

Directives are read line by line. In a parsed file, each line of each comment node counts, with comment markers and leading whitespace stripped. A block comment counts as several lines, and its line number is the comment's start line plus the line's offset. In a text file, each line counts after stripping leading whitespace and one leading `#`, `//`, `--`, `;` or `<!--`.

A line is a directive candidate only when it then starts with `slopr-ignore-`. Text that mentions `slopr-ignore-` later in a line is not a directive.

Grammar: `slopr-ignore-(next-line|file) <ids> -- <reason>`. `<ids>` is one or more rule IDs separated by commas, and each ID matches `[a-z0-9-]+/[a-z0-9-]+`.

`next-line` hides findings whose `start_line` is the first non-blank line after the end of the comment that holds the directive. In a text file, that is the first non-blank line after the directive's line. `file` hides findings anywhere in the file, wherever the directive sits. Directives apply to native and `fallow/*` findings alike.

A directive is invalid when it is malformed, has an empty reason, or names an ID that is neither a native rule nor `fallow/`-prefixed. A candidate that does not match the grammar, such as `slopr-ignore-next-lien`, is malformed. An invalid directive produces `suppress/invalid-directive` at warn on the directive's line. It suppresses nothing, even when some of its IDs are valid. That finding cannot be suppressed or configured. Unused directives are not reported.

## Baseline file

The file is JSON with 2-space indentation and a trailing newline:

```
{"schema_version": 1, "findings": [{"fingerprint", "rule", "path"}]}
```

Findings are sorted by path, rule and fingerprint, without duplicates.

`slopr baseline save <FILE>` runs the full `check` scan, with config, inline suppressions and fallow. It writes every finding, whatever its severity and the gate. It exits 0, or 2 or 3 on the usual errors.

`--baseline <FILE>` hides findings whose fingerprint is in the file. A missing or invalid file exits 2. Stale entries are ignored.
