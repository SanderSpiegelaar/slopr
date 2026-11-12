# Merged rule catalog

Native rule IDs use `family/name`. fallow findings keep fallow's IDs under `fallow/<rule>`.

Detection: AST = tree-sitter node match; Regex = regex over a comment or string node text; Diff = git diff; Graph = cross-file graph.
Cost: per-file CPU on top of parsing. FP = false-positive risk.

| Rule ID | Catches | Sources | Detection | Cost | FP | Decision |
|---|---|---|---|---|---|---|
| `comment/narrative` | "This function...", "It handles...", "First it...", "Then we..." | aislop `ai-slop/narrative-comment`; stop-ai-slop `vend/this-function-opener`; vibecheck `no-narrator-comments`; sloplint `narrator-comment`; slopsieve narration | AST comment + anchored phrase regex | low | medium | v0 warn |
| `comment/section-divider` | `// =====`, `# ---- Section ----` | aislop `DECORATIVE_SEPARATOR`; stop-ai-slop `vend/section-divider`; vibecheck `no-section-dividers`; sloplint `section-divider`; agent-slop-lint `information.section_comments`; SynthScan | AST comment + regex | low | low | v0 warn |
| `comment/step-numbered` | `// Step 1:`, `// Phase 2` | aislop `SECTION_HEADER`; stop-ai-slop `vend/step-numbered`; vibecheck `no-step-comments`; sloplint `step-comment` | AST comment + regex | low | low | v0 warn |
| `comment/agent-artifact` | "As an AI", "per the spec/plan", "I've updated", "Here is the updated code" in comments or strings | aislop `ai-slop/meta-comment`; stop-ai-slop `vend/ai-plan-narration`; OPK-002; vibecheck `no-ai-attribution`; sloplint `ai-generated-comment`, `apologetic-comment`; SynthScan AI Response Leakage | AST comment and string + phrase list | low | low | v0 error |
| `comment/untracked-todo` | TODO, FIXME, HACK, XXX without an issue reference | aislop `ai-slop/todo-stub`; stop-ai-slop `vend/generic-todo`; vibecheck `no-ai-todo`; sloplint `placeholder-comment`; OPK-003 | AST comment + regex, issue pattern configurable | low | low | v0 info |
| `text/invisible-unicode` | zero-width U+200B, U+200C, U+200D, U+2060, U+FEFF; bidi U+202A to U+202E, U+2066 to U+2069 | stop-ai-slop `vend/zero-width-chars`, `vend/bidi-controls` | byte scan, any file | trivial | very low | v0 error |
| `error/swallowed` | empty catch, `except: pass`, catch that only logs, `.catch(() => {})` | aislop `ai-slop/swallowed-exception`; vibecheck `no-empty-catch`, `no-pass-except`, `no-console-error-only`, `no-py-log-swallow`, `no-swallowed-promise`; sloplint `empty-error-handler`, `silent-exception`, `log-in-error-handler`; slopsieve; ai-slop-cleaner `error_swallow_auditor.py`; slop-guard `check_silent_catches`; slopgate | AST catch/except body | low | low to medium | v0 error |
| `error/redundant-rethrow` | catch that only rethrows the same error | aislop `ai-slop/redundant-try-catch`, `ai-slop/csharp-empty-catch-rethrow`; slop-guard `check_catch_log_reraise` | AST | low | low | v0 warn |
| `stub/placeholder-body` | `todo!()`, `unimplemented!()`, `raise NotImplementedError`, `pass`-only body, `throw new Error("not implemented")`, "implement this" | aislop `ai-slop/todo-stub`, `ai-slop/rust-todo-stub`, `ai-slop/empty-function`, `csharp-not-implemented`, `cpp-not-implemented`; vibecheck `no-stub-function`, `no-py-stub-function`; OPK-003; axonscanner claimed-only | AST | low | medium (abstract methods, protocols) | v0 warn; skip `@abstractmethod`, trait default bodies, `Protocol` |
| `test/tautological` | `expect(true).toBe(true)`, `assert True`, `assertEqual(1, 1)`, `assert!(true)` | aislop `ai-slop/tautological-test`; slopsieve test weakening | AST literal comparison in test files | low | low | v0 error |
| `debug/leftover` | `console.log/debug/info`, `debugger`, `print(` in library code, `dbg!`, `pdb.set_trace`, `breakpoint()` | aislop `ai-slop/console-leftover`, `python-print-debug`, `csharp-console-leftover`, `cpp-iostream-leftover`; vibecheck `no-console-pollution`, `no-py-print`; slopsieve | AST call match; skip tests, `bin/`, `scripts/`, files with a `main` entry | low | medium | v0 warn |
| `ts/unsafe-assertion` | `as any`, `as unknown as T`, `<any>x` | aislop `ai-slop/unsafe-type-assertion`, `double-type-assertion`; vibecheck `no-ts-any`, `no-double-assertion`; agent-slop-lint `structural.types.escape_hatches` | AST | low | low | v0 warn |
| `sec/hardcoded-secret` | `sk_live_`, `sk-`, `ghp_`, `github_pat_`, `AKIA`, `glpat-`, `xox[bp]-`, PEM private key headers | aislop `security/hardcoded-secret`; vibecheck `no-hardcoded-secrets`; OPK-001; slopsieve | regex on string nodes and raw text; redact evidence | low | low | v0 error |
| `comment/obvious` | comment restates the next statement | aislop `ai-slop/trivial-comment`; stop-ai-slop `vend/obvious-comment`; vibecheck `no-obvious-comments`; sloplint `obvious-comment`; rayonx | identifier-token overlap of comment and next statement | low | high | v1 warn, off by default until FP corpus passes |
| `comment/changelog` | "now uses X instead of Y", "refactored from", "we moved", "replaces the" | stop-ai-slop `changelog-marker`; aislop `CROSS_REFERENCE_PHRASES` | phrase list | low | medium | v1 warn |
| `comment/ai-vocab-density` | 3 or more distinct words like delve, pivotal, tapestry in one file's comments | stop-ai-slop `vend/ai-vocab-density`; SynthScan AI Slop Vocabulary | word set per file | low | medium | v1 info |
| `error/silent-recovery` | catch logs without the caught error, then continues | aislop `ai-slop/silent-recovery` | AST + identifier use | low | medium | v1 warn |
| `error/broad-catch` | bare `except:`, `except Exception` with pass-like body, `catch (Exception)` | aislop `python-bare-except`, `python-broad-except`, `java-broad-throws`, `csharp-broad-catch`; vibecheck `no-bare-except` | AST | low | medium | v1 warn (Python first) |
| `error/vague-message` | "Something went wrong", "An error occurred" in throw or raise | vibecheck `no-vague-error`; slop-guard `check_generic_error_messages` | AST + phrase | low | medium | v1 info |
| `suppress/unexplained` | `@ts-ignore`, `# type: ignore`, `eslint-disable`, `noqa`, `#[allow(...)]` without a reason | aislop `ai-slop/ts-directive`, `csharp-suppressed-warning`; vibecheck `no-type-ignore-blanket`; stop-ai-slop `vend/self-suppression`; fallow `added-suppression` | comment regex | low | low | v1 warn |
| `naming/generic` | `helper_1`, `data2`, `temp1`, `handleStuff`, `doTheThing` | aislop `ai-slop/generic-naming`; SynthScan magic placeholder names | AST identifiers + regex | low | medium | v1 info |
| `abstraction/thin-wrapper` | function that forwards its params unchanged to one call | aislop `ai-slop/thin-wrapper`; fallow `thin-wrapper` (React components only); Slop-Popper catalog 03 | AST | low | medium | v1 info |
| `config/hardcoded-endpoint` | environment URLs and provider project IDs in production code | aislop `ai-slop/hardcoded-url`, `hardcoded-id` | string regex, docstrings exempt | low | medium | v1 info |
| `complexity/function` | cyclomatic, cognitive, length, params, nesting over thresholds | aislop `complexity/*`; agent-slop-lint `structural.complexity.*`; vibecheck `no-god-function`, `no-deep-nesting`; aicomplex function-complexity | AST metric | low | low | JS/TS via fallow health; native v1 for Python, Rust, Go |
| `complexity/file-size` | god files | aislop `complexity/file-too-large`; agent-slop-lint `structural.god_module`; aicomplex god-files; rayonx | line count | trivial | low | JS/TS via fallow `complexity-file-lines`; native v1 |
| `dup/exact-block` | copy-pasted blocks | aislop `code-quality/duplicate-block`; agent-slop-lint `structural.duplication`; aicomplex duplication | normalized token windows + hash | medium | low | JS/TS via fallow dupes; native v1 for other languages |
| `pr/no-tests-changed` | 150+ source lines added, no test file touched | slopsieve; slopgate | Diff stats | trivial | medium | v1 info, diff mode only |
| `pr/oversized-diff` | more than 800 added lines or 15 files | slopsieve; slopgate | Diff stats | trivial | medium | v1 info, diff mode only |
| `test/weakened` | added `.skip`/`.only`, removed assertions or tests | slopsieve; fallow `weakened-tests` | Diff + AST | medium | low | JS/TS via `fallow audit`; native v2 |
| `dup/semantic` | same logic written differently, far apart | slopo | embeddings + cosine + clustering + distance boost | high, needs a model | medium, needs triage | JS/TS via `fallow similar-code` (opt-in); native v2 opt-in, clean-room |
| `deps/undeclared-import` | imports of packages not in the manifest (hallucinated) | aislop `ai-slop/hallucinated-import`; vibecheck `no-hallucinated-import-js`, `-py`; OPK-006; slopsieve | manifest + import parse | medium | low to medium | JS/TS via fallow `unlisted-dependencies`, `unresolved-imports`; Python native v2; registry lookups cut |
| `comment/hedging` | "should work", "might need to" | vibecheck `no-hedging-comments` | phrase | low | medium | later |
| `naming/stutter`, `naming/verbosity`, `naming/tersity` | identifier vocabulary discipline | agent-slop-lint `lexical.*` | AST identifiers | low | medium | later |
| `naming/symbol-collision` | one name defined in many places | aicomplex symbol-collision | Graph | medium | medium | later |
| `error/hidden-fallback` | `?? 0`, `\|\| []` that hide failures | aislop `ai-slop/hidden-fallback` | heuristic | medium | high | later |
| Dead code: unused files, exports, types, deps, members | | aislop (knip); aicomplex dead-exports, orphan-files; agent-slop-lint `structural.orphans` | Graph | | | via fallow for JS/TS; other languages use ruff, clippy, staticcheck; no native rule |
| Cycles and boundaries | | aicomplex import-cycles; agent-slop-lint `structural.deps`; aislop architecture | Graph | | | via fallow `circular-dependencies`, `boundary-violation`; no native rule |
| Security sinks: eval, innerHTML, SQL and shell concatenation | | aislop `security/*`; vibecheck `no-eval`, `no-innerhtml`, `no-sql-concat` | | | | JS/TS via `fallow security` (opt-in); no native rule (use Semgrep) |
| Halstead, NPath, CK class metrics, package distance, churn hotspots | | agent-slop-lint | AST + git | | | cut; hotspots via fallow health for JS/TS |
| Comment style policy: long, multi-line, markdown, citations, file header, cross-file refs, CJK noise | | stop-ai-slop only | | | | cut: style policy with one source |
| Vulnerable or unpinned deps, risky CI workflows | | aislop audits; OPK-004, OPK-005 | | | | cut: cargo-audit, npm audit, zizmor exist |
| PR description and author signals | | slopgate | GitHub API | | | cut: not a local check |
| Formatter and linter orchestration | | aislop format and lint engines | | | | cut: users keep their own linters |
| Score 0 to 100 | | aislop, aicomplex, vibecheck, rayonx | | | | cut: gate on severity plus baseline |

Overlap policy: a native rule marked "via fallow" leaves JS and TS out of its `langs`, so fallow alone covers those languages. Native rules that fallow lacks (comment, text, error, stub, test, debug, ts, sec rows above) always run.

## v0 language coverage

TS is TypeScript and Tsx. JS is JavaScript, parsed with the TSX grammar. PY is Python. RS is Rust. "Comment" means a comment node from `SourceFile::comments()`. It skips doc comments (`///`, `//!`, `/** */`) and Python docstrings, and gives the text with its markers stripped. "String" means a string node from `SourceFile::strings()`.

| Rule | Langs | Matches |
|---|---|---|
| `comment/narrative` | TS JS PY RS | Comment whose text starts (case-insensitive, after whitespace) with a phrase from the ported narrative list. |
| `comment/section-divider` | TS JS PY RS | Comment matching `^([=\-*#~_])\1{3,}` or `^([=\-*#~_])\1{2,}\s+.+?\s+([=\-*#~_])\2{2,}$`. Each run in the titled form repeats one character, so `-*- coding: utf-8 -*-` does not match. |
| `comment/step-numbered` | TS JS PY RS | Comment matching `(?i)^(step\|phase\|stage)\s+\d+\b`. |
| `comment/agent-artifact` | TS JS PY RS | Comment or string containing a phrase from the ported agent-artifact list, case-insensitive, on word boundaries. |
| `comment/untracked-todo` | TS JS PY RS | Comment containing `\b(TODO\|FIXME\|HACK\|XXX)\b` (case-sensitive) and no match of `todo_issue_pattern`. |
| `text/invisible-unicode` | all files | U+200B, U+200C, U+200D, U+2060, U+FEFF, U+202A to U+202E, U+2066 to U+2069. U+FEFF at byte offset 0 is a BOM and exempt. One finding per line, at the first hit, naming each code point found as `U+XXXX`. |
| `sec/hardcoded-secret` | all files | Raw text matching `sk_live_[0-9a-zA-Z]{20,}`, `sk-(proj-)?[A-Za-z0-9_-]{32,}`, `ghp_[A-Za-z0-9]{36}`, `github_pat_[A-Za-z0-9_]{82}`, `AKIA[0-9A-Z]{16}`, `glpat-[A-Za-z0-9_-]{20}`, `xox[bp]-[0-9A-Za-z-]{10,}`, `-----BEGIN ([A-Z]+ )?PRIVATE KEY-----`. The message shows the first 4 chars, then `...`, then the match length, never the whole match. |
| `error/swallowed` | TS JS PY RS | TS/JS: a `catch_clause` whose block has no statements, or only logging calls (callee rooted at `console`, `logger`, `log`) none of which passes the catch parameter. Also `.catch(f)` where `f` is a function with an empty body. PY: an `except_clause` whose block is only `pass` or `...`, or only logging calls (`logging.*`, `logger.*`, `log.*`, `print`) none of which passes the `as` name, `exc_info=True`, or is `.exception(...)`. RS: a match arm `Err(_) => {}` or `Err(_) => ()`, and an expression statement whose expression is a method call `.ok()`. |
| `error/redundant-rethrow` | TS JS PY | TS/JS: a catch block whose only statement is `throw <param>`. PY: an except block whose only statement is `raise`, or `raise <name>` where `<name>` is its `as` name. |
| `stub/placeholder-body` | TS JS PY RS | TS/JS: a function or method body whose only statement throws `new Error(<string>)` with the string matching `(?i)not implemented\|implement (me\|this)\|todo`. PY: a function body, ignoring its docstring, that is only `pass`, `...` or `raise NotImplementedError` (with or without a call), skipping `.pyi` files, functions decorated `abstractmethod` or `overload`, and methods of a class whose bases include `Protocol`, `ABC` or `ABCMeta`. RS: a fn body whose only expression is `todo!(...)` or `unimplemented!(...)`, skipping fns inside a `trait_item`. |
| `test/tautological` | TS JS PY RS | Test code only (`SourceFile::in_test`). TS/JS: `expect(A).toBe(B)`, `.toEqual(B)`, `.toStrictEqual(B)` with A and B the same literal text, `expect(true).toBeTruthy()`, `assert(true)`, `assert.ok(true)`, `assert.equal(A, B)` and `assert.strictEqual(A, B)` with the same literal. PY: `assert True`, `assert A == B` with the same literal, `self.assertTrue(True)`, `self.assertEqual(A, B)` with the same literal. RS: `assert!(true)`, `assert_eq!(A, B)` with the same literal. A literal is a boolean, number, string or null/None token. |
| `debug/leftover` | TS JS PY RS | Skipped in test code and under any `bin/` or `scripts/` dir. TS/JS: calls to `console.log`, `console.debug`, `console.info`, `console.trace`, `console.dir`, and `debugger` statements, skipping files that start with `#!`. PY: `print(...)` calls (skipping `__main__.py` and files with an `if __name__ == "__main__":` block), plus `pdb.set_trace()`, `breakpoint()`, `import pdb` everywhere. RS: `dbg!(...)` anywhere outside test code. `println!` is not flagged. |
| `ts/unsafe-assertion` | TS | `as_expression` whose type is `any`, an `as_expression` whose inner expression is itself `as unknown`, and a `type_assertion` with type `any`. |

All v0 rules are `default_on: true`. Default severities are the ones in the catalog's Decision column.

## v1 language coverage

The shorthand is the same as in v0 language coverage, plus GO for Go.

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
| `complexity/function` | PY RS GO | Metrics in Complexity and duplication metrics. One finding per function exceeding any limit, spanning the function, with each exceeded metric named as `name value/limit`. |
| `complexity/file-size` | PY RS GO | Non-blank lines over `file_max_lines`, file-level span. |
| `dup/exact-block` | PY RS GO | `Check::Project`. Metrics in Complexity and duplication metrics. |
| `pr/no-tests-changed`, `pr/oversized-diff` | any | `Check::Diff`, as in the cli.md diff mode section. |

## Complexity and duplication metrics

- Cyclomatic: 1, plus one for each `if`, `elif`/`else if`, loop (`for`, `while`, `loop`), match or switch arm after the first, `catch`/`except`, `&&`/`||`/`and`/`or`, and conditional expression.
- Cognitive (after SonarSource): +1 for each `if`, `else if`, `else`, loop, `catch`/`except`, switch or match, and each run of the same logical operator. Each nested `if`, loop, `catch` or match adds its nesting depth on top. Closures and nested functions add one level of nesting. No increment for recursion.
- Lines: non-blank lines from the function's first to last line.
- Params: declared parameters, excluding `self`, `&self`, `&mut self` and Go receivers.
- Nesting: the deepest chain of nested `if`, loop, match/switch, `try` and `catch`/`except` blocks.
- Duplication: tokens are the tree's leaf nodes minus comments. Identifiers become `$id`, literals become `$lit`. Hash every window of `dup_min_tokens` tokens per language, merge overlapping matching windows into maximal ranges, and report a range covering at least `dup_min_lines` lines. One finding per occurrence, with the message naming the other occurrences as `path:line`.

## Go coverage

Go joins all `comment/*` and `text/*` rules, `sec/hardcoded-secret`, and these:

- `error/swallowed`: `if err != nil {}` with an empty block.
- `stub/placeholder-body`: a function body whose only statement is `panic(<string>)`, with the string matching the TS/JS regex `(?i)not implemented|implement (me|this)|todo`.
- `test/tautological`: in `_test.go`, testify `assert.True(t, true)`, `require.True(t, true)`, and `assert.Equal(t, A, B)` with the same literal.
- `debug/leftover`: `fmt.Print*` and builtin `print`/`println`, skipping `package main` and test files.

Go does not join `error/redundant-rethrow` or `ts/unsafe-assertion`.
