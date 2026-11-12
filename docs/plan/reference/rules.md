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
| `error/hidden-fallback` | `?? 0`, `|| []` that hide failures | aislop `ai-slop/hidden-fallback` | heuristic | medium | high | later |
| Dead code: unused files, exports, types, deps, members | | aislop (knip); aicomplex dead-exports, orphan-files; agent-slop-lint `structural.orphans` | Graph | | | via fallow for JS/TS; other languages use ruff, clippy, staticcheck; no native rule |
| Cycles and boundaries | | aicomplex import-cycles; agent-slop-lint `structural.deps`; aislop architecture | Graph | | | via fallow `circular-dependencies`, `boundary-violation`; no native rule |
| Security sinks: eval, innerHTML, SQL and shell concatenation | | aislop `security/*`; vibecheck `no-eval`, `no-innerhtml`, `no-sql-concat` | | | | JS/TS via `fallow security` (opt-in); no native rule (use Semgrep) |
| Halstead, NPath, CK class metrics, package distance, churn hotspots | | agent-slop-lint | AST + git | | | cut; hotspots via fallow health for JS/TS |
| Comment style policy: long, multi-line, markdown, citations, file header, cross-file refs, CJK noise | | stop-ai-slop only | | | | cut: style policy with one source |
| Vulnerable or unpinned deps, risky CI workflows | | aislop audits; OPK-004, OPK-005 | | | | cut: cargo-audit, npm audit, zizmor exist |
| PR description and author signals | | slopgate | GitHub API | | | cut: not a local check |
| Formatter and linter orchestration | | aislop format and lint engines | | | | cut: users keep their own linters |
| Score 0 to 100 | | aislop, aicomplex, vibecheck, rayonx | | | | cut: gate on severity plus baseline |

Overlap policy: when fallow is enabled and the file is JS/TS, a native rule marked "via fallow" does not run. Native rules that fallow lacks (comment, text, error, stub, test, debug, ts, sec rows above) always run.
