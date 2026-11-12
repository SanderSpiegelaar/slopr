---
id: D-RULES-V1
title: Decide v1 rule coverage, metrics and Go coverage
kind: decision
depends_on: [D-RULES-V0, D-CONFIG]
docs: [docs/reference/rules.md#merged-rule-catalog, docs/reference/rules.md#v0-language-coverage, docs/reference/config.md#rule-settings]
touches: [docs/reference/rules.md]
creates: [docs/reference/rules.md#v1-language-coverage, docs/reference/rules.md#complexity-and-duplication-metrics, docs/reference/rules.md#go-coverage]
covers: [gap:D08b, gap:D19, gap:D20]
signoff: user
verify: for h in "v1 language coverage" "Complexity and duplication metrics" "Go coverage"; do grep -qx "## $h" docs/reference/rules.md || exit 1; done
---

## Goal

Pin coverage and formulas for every v1 rule, and say how Go joins the v0 rules.

## Scope

Add three sections after `## v0 language coverage`. The shorthand is the same, plus GO.

`## v1 language coverage`:

| Rule | Langs | Matches |
|---|---|---|
| `comment/obvious` | TS JS PY RS GO | A comment followed (next non-blank line) by a statement. Content words of the comment are its words, lowercased, split on camelCase and snake_case, minus a stopword list kept in the rule (a, an, the, to, of, and, or, in, on, for, with, this, that, it, is, be, we, then, now, here). Statement tokens are the statement's identifiers, split the same way. Fires when the comment has 2 to 8 content words and at least 80% of them are statement tokens. `default_on: false`. |
| `comment/changelog` | TS JS PY RS GO | Comment containing a phrase from the ported changelog list. |
| `comment/ai-vocab-density` | TS JS PY RS GO | Per file: fires once, with a file-level span (line 1 to last line), when the file's comments hold at least `vocab_min_distinct` distinct words from the ported vocabulary list. |
| `error/silent-recovery` | TS JS PY | A catch or except block that has a logging call not passing the caught name, at least one other statement, and no throw or raise. |
| `error/broad-catch` | PY | `except:`, `except Exception:` or `except BaseException:` whose block is only `pass`, `...`, `continue`, `return` or `return None`. |
| `error/vague-message` | TS JS PY RS GO | The first string argument of `new Error`/`Error`/any `*Error` constructor in a throw (TS/JS), of a raised exception (PY), of `panic!`/`bail!`/`anyhow!` (RS), or of `errors.New`/`fmt.Errorf` (GO), when it matches slopr's own vague list: `(?i)^(something went wrong\|an error occurred\|an unexpected error occurred\|error occurred\|unknown error\|oops)\b`. |
| `suppress/unexplained` | TS JS PY RS GO | TS/JS: `@ts-ignore`, or `eslint-disable`, `eslint-disable-line`, `eslint-disable-next-line` without ` -- <reason>`. PY: bare `# type: ignore` with no `[code]`, bare `# noqa` with no codes, `# pylint: disable` with no trailing `#` reason. RS: `#[allow(...)]` or `#![allow(...)]` with no `reason = "..."`. GO: `//nolint` with no trailing `// <reason>`. |
| `naming/generic` | TS JS PY RS GO | Declared names (variables, functions, parameters) matching `^(helper\|data\|temp\|tmp\|foo\|bar\|baz\|thing\|stuff\|obj\|val\|res\|result)_?\d+$` or `(?i)^(handle\|do\|process\|manage)_?(stuff\|things?\|it\|the_?thing)$`. |
| `abstraction/thin-wrapper` | TS JS PY RS GO | A function whose body is a single call, or `return` of a single call, passing exactly its own parameters in order, unchanged. Skips methods that call `super`, decorated functions, exported React components (a capitalized name returning JSX), and trait impl methods. |
| `config/hardcoded-endpoint` | TS JS PY RS GO | Strings outside test code and docstrings that are an `http(s)` URL whose host has a label `dev`, `stg`, `staging`, `qa`, `uat`, `prod` or `internal`, or that match `projects/[a-z][a-z0-9-]{4,28}[a-z0-9]` or `arn:aws:[a-z0-9-]+:[a-z0-9-]*:\d{12}:`. Hosts `localhost`, `127.0.0.1`, `example.com`, `example.org` are exempt. |
| `complexity/function` | PY RS GO | Metrics below, one finding per function exceeding any limit, spanning the function, with each exceeded metric named as `name value/limit`. |
| `complexity/file-size` | PY RS GO | Non-blank lines over `file_max_lines`, file-level span. |
| `dup/exact-block` | PY RS GO | `Check::Project`. Metrics below. |
| `pr/no-tests-changed`, `pr/oversized-diff` | any | `Check::Diff`, as in the cli.md diff mode section. |

`## Complexity and duplication metrics`:

- Cyclomatic: 1, plus one for each `if`, `elif`/`else if`, loop (`for`, `while`, `loop`), match or switch arm after the first, `catch`/`except`, `&&`/`||`/`and`/`or`, and conditional expression.
- Cognitive (after SonarSource): +1 for each `if`, `else if`, `else`, loop, `catch`/`except`, switch or match, and each run of the same logical operator. Each nested `if`, loop, `catch` or match adds its nesting depth on top. Closures and nested functions add one level of nesting. No increment for recursion.
- Lines: non-blank lines from the function's first to last line.
- Params: declared parameters, excluding `self`, `&self`, `&mut self` and Go receivers.
- Nesting: the deepest chain of nested `if`, loop, match/switch, `try` and `catch`/`except` blocks.
- Duplication: tokens are the tree's leaf nodes minus comments. Identifiers become `$id`, literals become `$lit`. Hash every window of `dup_min_tokens` tokens per language, merge overlapping matching windows into maximal ranges, and report a range covering at least `dup_min_lines` lines. One finding per occurrence, with the message naming the other occurrences as `path:line`.

`## Go coverage`. Go joins all `comment/*` and `text/*` rules, `sec/hardcoded-secret`, and these:

- `error/swallowed`: `if err != nil {}` with an empty block.
- `stub/placeholder-body`: a body that is only `panic(<string>)` matching the stub regex.
- `test/tautological`: in `_test.go`, testify `assert.True(t, true)`, `require.True(t, true)`, and `assert.Equal(t, A, B)` with the same literal.
- `debug/leftover`: `fmt.Print*` and builtin `print`/`println`, skipping `package main` and test files.

Go does not join `error/redundant-rethrow` or `ts/unsafe-assertion`.

## Out of scope

- Code.

## Acceptance criteria

- All three sections exist with the content above.
- `python3 scripts/check-tickets.py` passes.
