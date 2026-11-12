# Rust anti-slop CLI: build plan

Working name: `slopr` (placeholder, rename before publishing). Plan date: 2026-11-11. Author: Recon (read-only analysis). Nothing in this plan has been built or run.

## 1. Goal and bottom line

Goal: one Rust binary that flags the code-quality problems AI coding agents leave behind, combining the deterministic checks found in the source repos below, and exposing every fallow check for TypeScript and JavaScript projects.

Bottom line:

- Build a single-crate Rust CLI. v0 ships 13 native, deterministic rules on tree-sitter (comment slop, swallowed errors, stubs, tautological tests, debug leftovers, unsafe TS assertions, invisible Unicode, hardcoded secrets) plus fallow passthrough, with human, JSON and SARIF output and CI exit codes.
- Integrate fallow by running a separately installed, version-checked `fallow` binary as a subprocess and ingesting its SARIF output. Do not link the fallow Rust crates and do not use the NAPI bindings. Reason: fallow documents its CLI and JSON output as semver-stable, and documents every Rust crate (including `fallow-api`) as "not supported external semver APIs" that "may change in a minor release" (`fallow-rs/fallow:docs/backwards-compatibility.md`, section "What is not covered").
- Do not re-implement what fallow already does well for JS/TS (dead code, unlisted and unresolved imports, duplication, complexity, cycles, boundaries, weakened tests). Native rules target slop signals fallow does not cover and languages fallow does not parse.
- LLM or embedding checks are optional and come last (phase v2).

## 2. Assumptions (no blocking question was asked)

- Target implementation language is Rust (stable toolchain).
- Open-source friendly licensing is wanted. Recommended project license: `MIT OR Apache-2.0`, which allows porting MIT and Apache-2.0 sources with attribution.
- Must-have: deterministic local checks and fallow passthrough. Nice-to-have: LLM or embedding checks.
- No telemetry, no network access during `check`.
- v0 languages: TypeScript, TSX, JavaScript, Python, Rust. Go joins in v1.

## 3. Sources

### 3.1 Collection

- `rafal-qa/slopo`: 1 repo.
- `github.com/scanaislop` org: 7 public repos (API listing on 2026-11-11).
- `github.com/topics/ai-slop`: 546 repos returned by the GitHub search API (`q=topic:ai-slop`) on 2026-11-11. Most are prose humanizers, UI design skills, browser extensions and games. Filtered on description keywords for code quality, lint, static analysis, dead code, duplication, comments, PR review, then dropped no-code skill packs and unrelated projects.
- fallow: `fallow-rs/fallow` plus `https://fallow.tools/llms-full.txt` (docs source commit `256a68ca`).

Totals: 549 unique candidate repos (1 slopo, 7 org, 546 topic; 5 org repos also carry the topic). 19 analyzed as rule sources, plus fallow as the integration target, 20 in total. 530 dropped: 525 topic repos (prose humanizers, UI design skills, browser extensions, games, no detector code) and 5 scanaislop org repos: `flashwave` (unrelated Expo strobe-light app, no license), `aislop-py` (pipx wrapper of the same CLI), `homebrew-tap` (packaging), `mintlify-docs` (docs only), `.github` (profile). Code was read from shallow clones in `/workspace`, which were deleted after analysis.

### 3.2 Repos analyzed, ranked by relevance then maintenance

Data from the GitHub API on 2026-11-11. "Pushed" is the last push to any branch.

| Rank | Repo | Lang | License | Pushed | Open issues | Stars | Detection approach | Why it matters |
|---|---|---|---|---|---|---|---|---|
| 1 | fallow-rs/fallow | Rust | MIT | 2026-11-11 | 18 | 5338 | oxc AST, module graph, token clone detection, git churn | Integration target. ~70 issue types for JS/TS. |
| 2 | scanaislop/aislop | TypeScript | MIT | 2026-11-07 | 19 | 739 | regex on comment/string-masked source, plus orchestration of oxlint, biome, knip, ruff, clippy, golangci-lint and others | Largest deterministic slop rule set (60+ rule IDs). |
| 3 | rafal-qa/slopo | Python | AGPL-3.0-or-later | 2026-10-31 | 0 | 830 | tree-sitter function and block units, embeddings via LiteLLM, cosine pairs, transitive clustering, distance boost | Semantic duplicate detection, the hardest slop to find. Clean-room only. |
| 4 | JordanGunn/agent-slop-lint | Python | Apache-2.0 + NOTICE | 2026-11-11 | 1 | 44 | tree-sitter metric kernels, ripgrep, fd, git churn | Structural and lexical metrics with published thresholds and waivers with reasons. |
| 5 | WhiteBite/stop-ai-slop | JavaScript | MIT | 2026-11-08 | 1 | 7 | line and comment regex, multilingual phrase lists, detekt port | Most complete comment-slop catalog (19 rules), invisible Unicode checks. |
| 6 | yuvrajangadsingh/vibecheck | TypeScript | FSL-1.1-Apache-2.0 | 2026-11-05 | 0 | 27 | custom lexer plus regex | 40 rules overlapping aislop. Source-available, not open source. Ideas only. |
| 7 | princejain756/slopsieve | JavaScript | MIT | 2026-10-24 | 0 | 0 | unified diff parsing, weighted signals | Diff and PR-level signals (test weakening, deleted tests, oversized diff). |
| 8 | jackmcpickle/aicomplex | TypeScript | MIT in package.json, no LICENSE file | 2026-11-10 | 1 | 2 | web-tree-sitter, cross-file index | Findability metrics (symbol collision, barrel depth, cross-file connectivity). License unverified. |
| 9 | dannote/sloplint | TypeScript | MIT | 2026-02-04 | 0 | 4 | ast-grep (tree-sitter) comment and catch nodes | Shows comment rules on real AST nodes across 9 languages. Stale. |
| 10 | NhanAZ-Drops/OpenPolicyKit | TypeScript | MIT | 2026-06-10 | 0 | 0 | regex | AI credentials, prompt artifacts, placeholder code. |
| 11 | marcoramilli/SynthScan | Python | MIT | 2026-10-24 | 0 | 2 | regex patterns from a markdown file, Python `ast` | Large phrase and identifier pattern lists. |
| 12 | TreeStan/slopgate | Python | MIT | 2026-09-17 | 0 | 0 | GitHub API, diff stats | PR reviewability signals. Mostly out of scope (needs API). |
| 13 | AI-Labs-Pvt-Ltd-Nepal/axonscanner | Python | Apache-2.0 | 2026-08-07 | 0 | 1 | Python `ast`, call graph, test reach | "Claimed-only" modules (stubs that look finished). |
| 14 | devilbhai/ai-slop-cleaner | Python | MIT | 2026-10-31 | 0 | 0 | regex, npm and PyPI registry HTTP lookups | Ghost package lookup. Network-based, so not adopted. |
| 15 | trisouro/slop-guard | Python | none | 2026-07-03 | 0 | 2 | Python `ast` and regex | Catch-log-reraise, generic error messages. No license: ideas only. |
| 16 | gretchenfitze/anti-slop | JavaScript | MIT | 2026-10-22 | 0 | 4 | agent skills that wire tsc, oxlint, knip, jscpd, Stryker | Rollout method (warn on changed files first, gate last). |
| 17 | scanaislop/skills | Markdown | MIT | 2026-11-01 | 0 | 5 | agent skill | Agent integration pattern for a slop CLI. |
| 18 | J7Forge/Slop-Popper | Markdown | MIT | 2026-11-11 | 0 | 0 | agent skill, catalog prose | Fix catalog (13 categories). No detector code. |
| 19 | dmdjr1409/rayonx | Python | MIT | 2026-09-16 | 0 | 0 | regex | Trivial comments, god files. Thin. |
| 20 | amanhammadK/AI-Slop-Linter-for-code | Python | MIT | 2026-05-14 | 0 | 0 | regex, single file | Placeholder and hallucination phrases. Thin. |

### 3.3 Per-repo notes (code read, paths cited)

fallow (`fallow-rs/fallow`, MIT)
- Rust workspace, 20 crates, version 3.42.0, `rust-version = "1.97"`, `edition = "2024"` (`Cargo.toml`). Parses with oxc 0.157 (`oxc_parser`, `oxc_semantic`), CSS with `lightningcss = "=1.0.0-alpha.72"` and `cssparser = "=0.37.0"` (exact pins, `Cargo.toml`).
- Published on crates.io: `fallow-cli`, `fallow-engine`, `fallow-api`, `fallow-core`, `fallow-types`, `fallow-output`, `fallow-mcp`, `fallow-license` and others, all 3.42.0 (crates.io API, 2026-11-10).
- `fallow-core` is "internal"; its entry points are deprecated (`docs/fallow-core-migration.md`, `crates/core/src/lib.rs` deprecation notes). All Rust crate APIs are excluded from semver (`docs/backwards-compatibility.md`, "Rust crate APIs").
- Stable surfaces: config format, rule names, JSON output with per-envelope `schema_version` (dead-code 10, combined 15 per llms-full.txt), CLI flags. SARIF details "beyond what the SARIF spec requires" are not covered.
- Output formats: human, json, sarif, compact, markdown, codeclimate. Exit codes: 0 no enforced gate failed, 1 gate failed, 2 fatal error (llms-full.txt, "Exit codes"). Bare `fallow --format json` exits 0 on findings unless `--fail-on-issues`.
- `fallow schema` prints a capability manifest (`manifest_version` "1") listing every command, issue type and MCP tool. Use it to list all fallow checks without hand-maintaining a list.
- MCP tools wrap CLI commands (for example `analyze` is `fallow dead-code --format json`).
- NAPI package `@fallow-cli/fallow-node`: "The API is new. Until it is stable, a minor version can have breaking changes."
- Telemetry is opt-in (`fallow telemetry` docs). The paid part (runtime coverage, `fallow-license`) is not needed for static checks.
- Release cadence: v3.38.1 on 2026-10-31 to v3.42.0 on 2026-11-10, five releases in ten days.
- Related: `fallow-rs/cargo-fallow` 0.3.0 (MIT) analyzes Rust workspaces (unused deps, over-public items, dupes, complexity). Seen, not analyzed in depth.

aislop (`scanaislop/aislop`, MIT, v0.20.0)
- Six engines: format, lint, code-quality, ai-slop, architecture, security (`src/config/schema.ts`, `docs/rules.md`).
- Native slop rules are regex over source with strings and comments masked by hand-written per-language maskers (`src/utils/source-masker*.ts`), not an AST. Example: narrative comments use anchored phrase regexes (`src/engines/ai-slop/narrative-comments-patterns.ts`). Duplicate blocks use normalized 10-line windows with at least 7 distinct lines, JS/TS only (`src/engines/code-quality/duplicate-block.ts`).
- Shells out to oxlint, biome, knip, ruff, golangci-lint, clippy, cargo fmt, cppcheck, clang-tidy, Roslynator, jb inspectcode, expo-doctor, npm/pip/cargo audits (`src/engines/lint/*`, `src/engines/format/*`, `src/engines/security/audit*.ts`).
- Config `.aislop/config.yml` (zod schema), defaults `maxFunctionLoc 80`, `maxFileLoc 400`, `maxNesting 5`, `maxParams 6`.
- Output: terminal, `--json`, `--sarif` (2.1.0), score 0 to 100, `ci` command with score gate, baseline, MCP server (`src/mcp.ts`), agent hooks for 10 agents (`src/hooks/install/*`).
- Telemetry on by default outside CI; opt out with `AISLOP_NO_TELEMETRY=1` or `DO_NOT_TRACK=1` (`docs/telemetry.md`).
- Documented false-positive handling: Python docstring URL exemption, `exc_info` exemption, call-wrapped mutable defaults (`docs/rules.md`, "Rule notes").

slopo (`rafal-qa/slopo`, AGPL-3.0-or-later, v0.9.1)
- Indexes functions and blocks with tree-sitter for 15 languages (`src/slopo/indexing/parsing/lang/*.py`), units above `body_node_count_threshold` 20 and `block_node_count_threshold` 60 (`src/slopo/config.py`).
- Embeds with any LiteLLM provider or Ollama (`src/slopo/embedding/*`), recommends Jina code embeddings at 256 dims (`README.md`).
- Cosine similarity pairs above 0.93 (analyze) or 0.92 (review), transitive clustering (`src/slopo/result/clustering.py`), rerank boost for distance across directories (max 0.15 over 8 hops) and within a file (max 0.1, per 250 lines) (`src/slopo/result/boost.py`), overlap removal (`src/slopo/result/overlap.py`), cluster hash for ignore file (`src/slopo/result/identity.py`).
- Output: Markdown reports for agents, `review` mode over git changes, Claude Code and Codex skills (`src/slopo/agent/configs/*`). No JSON or SARIF.
- Needs an embedding API key or local model. Not deterministic across providers.

agent-slop-lint (`JordanGunn/agent-slop-lint`, Apache-2.0 with NOTICE)
- Rule IDs (`docs/rules/README.md`): `structural.complexity.{cyclomatic,cognitive,npath}`, `structural.class.{complexity,coupling,inheritance.depth,inheritance.children}`, `structural.hotspots`, `structural.packages`, `structural.deps`, `structural.local_imports`, `structural.redundancy`, `structural.types.{sentinels,hidden_mutators,escape_hatches}`, `structural.duplication`, `structural.god_module`, `structural.orphans`, `information.{volume,difficulty,magic_literals,section_comments}`, `lexical.{stutter,verbosity,tersity}`.
- Defaults: CCX 10, CogC 15, NPath 400; profiles default, lax, strict (`src/cli/slop/config.py`).
- Config `.slop.toml` with `[[waivers]]` that require `id`, `path`, `rule`, `reason` (`.slop.toml`). Exit 0 clean, 1 violations, 2 error. Output human, json, quiet (`README.md`).
- Requires ripgrep and fd binaries (`src/cli/slop/_util/doctor.py`).

stop-ai-slop (`WhiteBite/stop-ai-slop`, MIT, v2.1.2)
- 19 rule IDs (`detekt-rules/rule-catalog.json`, `README.md`): `multi-line-comment`, `changelog-marker`, `long-comment`, `vend/step-numbered`, `vend/section-divider`, `vend/markdown-in-comment`, `vend/this-function-opener`, `vend/file-summary-header`, `vend/generic-todo`, `vend/ticket-ref`, `vend/cross-file-ref`, `vend/obvious-comment`, `vend/ai-plan-narration`, `vend/ai-vocab-density`, `vend/research-citation`, `vend/self-suppression`, `vend/cjk-noise`, `vend/zero-width-chars`, `vend/bidi-controls`.
- Phrase lists in ru, en, de, fr, es. Zero dependencies, pre-commit hook, MCP, detekt port, autofix for some rules. Has a detection benchmark corpus (`detbench-corpus.json`) and FP audit self-test (`skill/scripts/selftest/checks/fp-audit.mjs`).

vibecheck (`yuvrajangadsingh/vibecheck`, FSL-1.1-Apache-2.0)
- 40 rule IDs in `src/rules/*.ts` (for example `no-empty-catch`, `no-narrator-comments`, `no-hallucinated-import-js`, `no-ts-any`, `no-hedging-comments`, `no-vague-error`, `no-stub-function`). Custom lexer (`src/lexer.ts`), SARIF (`src/sarif.ts`), baseline and fingerprints, MCP.
- FSL restricts competing use for two years per release. Treat as ideas only.

slopsieve (`princejain756/slopsieve`, MIT)
- Diff-scoped weighted signals (`README.md`, `src/checks.js`): possible secret 40, test weakening 30, debug leftovers 25, test file deleted 25, import of missing local file 20, empty catch 20, trivial dependency 20, feature code with zero test changes 15, commented-out code 15, narration comments 10, exported but unreferenced 10, whitespace churn 10, oversized diff 10, vague util sprawl 5, no linked context 5.

sloplint (`dannote/sloplint`, MIT)
- ast-grep NAPI with tree-sitter grammars for 9 languages (`src/languages.ts`). Rules: `ai-generated-comment`, `apologetic-comment`, `empty-error-handler`, `log-in-error-handler`, `narrator-comment`, `obvious-comment`, `placeholder-comment`, `section-divider`, `silent-exception`, `step-comment` (`src/rules/*.ts`). Per-language tool-directive regex to skip `eslint-disable`, `noqa` and similar.

OpenPolicyKit (`NhanAZ-Drops/OpenPolicyKit`, MIT)
- `OPK-001` AI credentials, `OPK-002` prompt artifacts, `OPK-003` placeholder code, `OPK-004` unpinned versions, `OPK-005` risky CI workflows, `OPK-006` hallucinated dependencies, `OPK-007` large generated files (`src/rules/opk-*.ts`).

aicomplex (`jackmcpickle/aicomplex`)
- Analyzers in `src/analyze/`: barrel-depth, crap, cross-file-connectivity, dead-exports, duplication, entrypoints, error-masking, function-complexity, god-files, import-cycles, orphan-files, symbol-collision. web-tree-sitter for TS, JS, Python, Go. Slop score 0 to 100.
- License: `package.json` says MIT, GitHub reports no license, no LICENSE file in the repo. Unverified. Ideas only until a LICENSE file exists.

Smaller sources
- SynthScan: pattern categories in `patterns/synthetic_patterns.md` (AI response leakage, magic placeholder names, overly generic function names, excessive try-catch wrapping, decorative separators, TODO padding).
- slop-guard (no license): `lint/slop_lint.py` checks `check_catch_log_reraise`, `check_generic_error_messages`, `check_silent_catches`, `check_trivial_docstrings`, `check_speculative_logging`, `check_test_files_outside_tests`.
- ai-slop-cleaner: `scripts/ghost_package_hunter.py` queries `registry.npmjs.org` and PyPI. Network dependency, rejected for a deterministic check.
- axonscanner: verification ladder from asserted to claimed-only per module (`src/axonscanner/scanner.py`).
- slopgate: PR description, linked issue, tests, author signals via GitHub API (`slopgate.py`).
- rayonx, AI-Slop-Linter-for-code: regex trivial-comment and placeholder lists, nothing not covered above.

## 4. Merged rule catalog

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

## 5. fallow integration

### 5.1 Options compared

| Option | Stability | Version pinning | License | Build complexity | Startup cost | Verdict |
|---|---|---|---|---|---|---|
| Subprocess: `fallow <cmd> --format sarif` (and JSON where needed) | CLI and JSON are semver-covered; SARIF fields required by the spec are stable | Check `fallow --version` against a semver range; users upgrade fallow on their own schedule | MIT, no linking | None for slopr; user installs fallow via npm, cargo, Homebrew, Docker or release binary | One process per command; fallow keeps a cache in `.fallow/` | Recommended |
| Link `fallow-api` or `fallow-engine` 3.42 crates | Not semver-covered; may break in any minor | Requires `=3.42.x` exact pin, re-port on every minor (5 releases in 10 days) | MIT | MSRV 1.97, edition 2024, oxc 0.157, exact pins on `lightningcss` alpha and `cssparser`; heavy compile; conflicts if slopr adds its own oxc | None | Rejected |
| NAPI `@fallow-cli/fallow-node` | "API is new", minor can break | npm | MIT | Embeds a Node runtime in a Rust CLI | Node boot | Rejected |
| fallow MCP server | MCP tools wrap the CLI JSON | | MIT | MCP client in slopr | Extra handshake | Rejected for slopr; agents can call fallow MCP directly |

Recommendation: run a separately installed fallow binary as a subprocess, accept versions `>=3.42, <4`, read SARIF for findings and `fallow schema` for the list of checks.

### 5.1.1 Decision record (2026-11-11)

Decision: slopr integrates fallow as a subprocess and reads its SARIF output. Supported fallow versions are `>=3.42, <4`. The user installs fallow separately (npm, cargo, Homebrew, Docker or release binary). slopr checks the fallow version at startup and, when fallow is missing or out of range, stops with a clear error that names the found version (if any), the required range, and an install hint such as `npm install -D fallow@^3.42` or `cargo install fallow-cli --version ^3.42`. Approved by Sander on 2026-11-11.

Rejected: shipping slopr as one self-contained binary with fallow linked in. It would need an exact pin on fallow's Rust crates, which are not semver-stable, and a re-port on every fallow minor release.

### 5.2 Behavior

1. Enablement. `fallow.enabled = "auto"` (default) requires fallow when the scan root, or a workspace under it, has a `package.json`. `"on"` always requires it. `"off"` skips fallow and its startup check.
2. Locate binary in order: `--fallow-bin`, config `fallow.bin`, `<root>/node_modules/.bin/fallow`, `PATH`.
3. Startup version check, before any analysis: run `fallow --version`, parse with `semver`, require `fallow.version` (default `>=3.42, <4`). When fallow is required and is missing or out of range, exit 3 with an error naming the found version, the required range and an install hint (`npm install -D fallow@^3.42`, `cargo install fallow-cli --version ^3.42`). `--fallow off` is the escape hatch.
4. Full scan runs, from the project root: `fallow dead-code --format sarif --quiet`, `fallow dupes --format sarif --quiet`, `fallow health --format sarif --quiet`. Optional commands from config `fallow.commands`: `security`, `architecture`, `flags`.
5. Diff scan (`--changed-since <ref>`) runs `fallow audit --base <ref> --format sarif --quiet` instead, which includes the diff-risk rules (`weakened-tests`, `added-suppression`, `test-only-branch`).
6. Parse each SARIF log with `serde_json::Value`, read only spec-required fields: `runs[].results[].ruleId`, `level`, `message.text`, `locations[0].physicalLocation.artifactLocation.uri`, `region.startLine`, `startColumn`, `endLine`. Map level `error` to Error, `warning` to Warn, `note` and `none` to Info. Prefix rule IDs with `fallow/` unless already prefixed.
7. SARIF output from slopr appends fallow's runs verbatim as extra `runs[]` entries, so GitHub code scanning shows fallow's own rule metadata. Human and JSON outputs use the mapped findings.
8. Exit codes from fallow: 0 and 1 are both success for slopr (slopr applies its own gate from finding severities). 2 or a signal is a tool failure: slopr exits 3.
9. Config is not translated. fallow reads its own `.fallowrc.json`, `.fallowrc.jsonc` or `fallow.toml`. slopr passes `fallow.args` through unchanged.
10. `slopr rules --all` runs `fallow schema` and lists every fallow issue type next to native rules.

Phase 0 spike must confirm on a fixture: that each of the three commands emits SARIF with locations for every finding kind (duplication results may carry several locations), and the wall time of three commands versus one bare `fallow --format json` run. If SARIF drops findings that JSON has, switch step 4 to the combined JSON envelope (schema_version 15) and map its arrays.

## 6. Architecture

### 6.1 Shape

One binary crate, modules not crates. Split into a workspace only when compile time demands it.

Data flow: config, walk, parse, run native rules, run fallow (in parallel with native rules on a separate thread), apply suppressions and baseline, sort, report, exit code.

### 6.2 Folder tree

```
slopr/
  Cargo.toml
  LICENSE-MIT
  LICENSE-APACHE
  THIRD_PARTY_NOTICES.md
  src/
    main.rs            clap CLI, exit codes
    config.rs          slopr.toml model and loading
    walk.rs            ignore::WalkBuilder, language detection by extension
    source.rs          SourceFile: text, tree-sitter Tree, line index
    finding.rs         Finding, Severity, Span, Fingerprint
    rules/
      mod.rs           RULES table, run_rules
      comments.rs
      errors.rs
      stubs.rs
      tests.rs
      debug.rs
      types.rs
      text.rs          invisible unicode, secrets
      phrases.rs       phrase lists, each list cites its source and license
    fallow.rs          locate, version check, run, SARIF to Finding
    diff.rs            git diff --unified=0 parsing for --changed-since and --staged
    suppress.rs        inline directives and baseline file
    report/
      human.rs
      json.rs
      sarif.rs
  skill/
    SKILL.md
  tests/
    rules.rs           fixture harness
    cli.rs             end-to-end snapshots
    fallow_contract.rs
    fixtures/
      rules/<rule-id>/bad.<ext>
      rules/<rule-id>/good.<ext>
      repos/ts-app/    package.json, planted fallow issues
      repos/py-lib/
      repos/rust-crate/
```

### 6.3 Dependencies (versions from crates.io on 2026-11-11)

| Crate | Version | License | Use |
|---|---|---|---|
| clap (derive) | 4.6.7 | MIT OR Apache-2.0 | CLI |
| serde, serde_json | 1.0.229, 1.0.154 | MIT OR Apache-2.0 | config, JSON, SARIF |
| toml | 1.2.1 | MIT OR Apache-2.0 | config |
| ignore | 0.4.34 | Unlicense OR MIT | gitignore-aware walk |
| globset | 0.4.21 | Unlicense OR MIT | overrides, excludes |
| rayon | 1.12.0 | MIT OR Apache-2.0 | parallel per-file rules |
| regex | 1.13.1 | MIT OR Apache-2.0 | phrase and secret patterns |
| aho-corasick | 1.1.5 | Unlicense OR MIT | fast phrase pre-filter |
| tree-sitter | 0.27.2 | MIT | parsing |
| tree-sitter-typescript, -python, -rust, -go | 0.23.2, 0.25.0, 0.24.2, 0.25.0 | MIT | grammars (Go in v1) |
| semver | 1.x (check at build) | MIT OR Apache-2.0 | fallow version range |
| anyhow, thiserror | 1.0.104, 2.0.21 | MIT OR Apache-2.0 | errors |
| insta, assert_cmd (dev) | 1.49.0, 2.2.2 | Apache-2.0, MIT OR Apache-2.0 | snapshots, CLI tests |

Not used, with reason: `serde-sarif` 0.8.0 (last release 2025-05, and slopr needs raw passthrough of fallow runs anyway; a 60-line struct set is enough); `ast-grep-core` 0.51.2 (good option if rules move to declarative patterns; not needed for v0 Rust-function rules); `oxc_parser` (fallow already covers JS/TS semantics; tree-sitter gives one API across languages); `rmcp` 3.9.0 (MCP server deferred to v2).

Phase 0 must verify that the four grammar crates build against tree-sitter 0.27 through the `tree-sitter-language` `LanguageFn` interface (`parser.set_language(&tree_sitter_rust::LANGUAGE.into())`).

### 6.4 Core types

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity { Info, Warn, Error }

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang { TypeScript, Tsx, JavaScript, Python, Rust, Go }

#[derive(Clone, Copy, Serialize)]
pub struct Span { pub start_line: u32, pub start_col: u32, pub end_line: u32, pub end_col: u32 }

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Origin { Native, Fallow { version: String, command: String } }

#[derive(Clone, Serialize)]
pub struct Finding {
    pub rule: String,            // "error/swallowed" or "fallow/unused-exports"
    pub severity: Severity,
    pub path: String,            // repo-relative, forward slashes
    pub span: Span,
    pub message: String,
    pub fingerprint: String,     // hex(sha256(rule, path, normalized snippet))[..16]
    pub origin: Origin,
}

pub struct SourceFile { pub path: String, pub lang: Lang, pub text: String, pub tree: tree_sitter::Tree, pub is_test: bool }

pub struct Hit { pub span: Span, pub message: String }

pub struct Rule {
    pub id: &'static str,
    pub summary: &'static str,
    pub default_severity: Severity,
    pub langs: &'static [Lang],     // empty = all files, including unparsed text
    pub via_fallow_for_js: bool,    // skip on JS/TS when fallow ran
    pub check: fn(&SourceFile, &RuleSettings, &mut Vec<Hit>),
}

pub static RULES: &[Rule] = &[ /* one entry per native rule */ ];
```

Rules are a static table of plain functions. No trait objects, no plugin loading, no rule DSL in v0. Adding a rule is one function, one table row, one fixture directory.

Fingerprint uses the trimmed text of the flagged lines, not line numbers, so it survives line shifts (same idea as fallow `finding_id` and vibecheck `src/fingerprint.ts`).

### 6.5 CLI

```
slopr check [PATHS]...
    --format human|json|sarif      default human
    --changed-since <REF>          only report findings on changed lines; fallow runs `audit --base`
    --staged                       only staged changes
    --baseline <FILE>              hide findings whose fingerprint is in the file
    --fallow auto|on|off           override config
    --fallow-bin <PATH>
    --max-severity-exit <LEVEL>    default error; set warn to fail on warnings
slopr rules [--all]                native rules; --all adds fallow issue types from `fallow schema`
slopr explain <RULE>               summary, examples from fixtures, sources
slopr baseline save <FILE>         write current fingerprints
slopr init                         write slopr.toml with defaults
slopr skill                        print skill/SKILL.md for agents
```

Exit codes: 0 no finding at or above the gate severity after suppressions and baseline; 1 gate failed; 2 usage or config error; 3 required external tool missing or failed.

### 6.6 Config: `slopr.toml`

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

Inline suppression: `slopr-ignore-next-line <rule-id> -- <reason>` and `slopr-ignore-file <rule-id> -- <reason>`. Rule ID and reason are required. A directive without them is itself a finding `suppress/invalid-directive` (aislop `ai-slop/unknown-directive` shows the need).

### 6.7 Output

- human: grouped by file, `path:line:col severity rule message`, summary counts by rule.
- json: `{ "schema_version": 1, "version", "findings": [Finding], "summary": {...}, "tools": [{"name": "fallow", "version", "status"}] }`. Bump `schema_version` on any rename or removal, following fallow's policy.
- sarif: SARIF 2.1.0 with one run for slopr (tool.driver.rules from `RULES`, `partialFingerprints` from `fingerprint`) plus fallow's runs appended.

### 6.8 CI and coding-agent integration

- GitHub Actions: `slopr check --format sarif > slopr.sarif` then `github/codeql-action/upload-sarif`. Gate step: `slopr check --changed-since origin/${{ github.base_ref }}`.
- pre-commit: `.pre-commit-hooks.yaml` entry running `slopr check --staged`.
- Agents: `skill/SKILL.md`, printed by `slopr skill`. It tells the agent to run `slopr check --changed-since HEAD --format json` before handing back, fix error findings, never add a suppression without a reason, and treat `dup/semantic` clusters as candidates to verify (the slopo skills pattern, `src/slopo/agent/configs/*`). No MCP server in v0 or v1; agents can use fallow's MCP for JS/TS depth.

## 7. Test strategy

- Fixture harness (`tests/rules.rs`): for each `tests/fixtures/rules/<rule-id>/`, every line in `bad.*` that ends with a marker comment `expect: <rule-id>` must produce exactly that finding, and `good.*` must produce none. One harness, no per-rule test code.
- Seed `good.*` with the documented false-positive cases from the sources: aislop "Rule notes" (Python docstring URLs, `exc_info=True`, call-wrapped mutable defaults), abstract methods for `stub/placeholder-body`, CLI `main` files for `debug/leftover`, tool directives (`eslint-disable`, `noqa`) for comment rules (sloplint `src/languages.ts`).
- CLI snapshots (`tests/cli.rs`, insta + assert_cmd): human, json, sarif output on `fixtures/repos/*`, and each exit code path (0, 1, 2, 3).
- SARIF validity: validate output against the SARIF 2.1.0 JSON schema in a dev-only test.
- fallow contract test (`tests/fallow_contract.rs`, skipped unless `FALLOW_BIN` is set): CI installs `fallow@3.42.0` from npm, runs slopr on `fixtures/repos/ts-app` with planted issues (unused export, unlisted dependency, duplicated block, function over the cognitive ceiling, circular import) and asserts the matching `fallow/*` rule IDs appear. A weekly CI job runs the same test against the latest fallow to catch drift early.
- False-positive corpus: a script runs `slopr check --format json` over pinned commits of five mature repos (for example ripgrep, flask, zod, cobra, fallow itself) and prints findings per 1,000 lines per rule. A rule that exceeds 1 finding per 1,000 lines on mature code stays off by default. This mirrors aislop `docs/oss-benchmark.md` and stop-ai-slop `detbench-corpus.json`.
- Determinism: run twice, byte-compare JSON output.

## 8. Roadmap

Phase 0, spike (1 to 2 days). Build nothing permanent. Confirm the four tree-sitter grammars compile with tree-sitter 0.27. Run fallow 3.42 on a fixture TS repo with `dead-code`, `dupes`, `health` as SARIF and the bare run as JSON; record which findings and locations each format carries and the wall time. Output: a short note that fixes the SARIF versus JSON decision in section 5.2.

v0, smallest useful tool (about 1 to 2 weeks). `slopr check` on TS, TSX, JS, Python, Rust. The 13 v0 rules from section 4. fallow passthrough with the startup version check and install-hint error (decision 5.1.1). human, json, sarif. Inline suppressions with reasons. Exit codes. Fixture harness, CLI snapshots, fallow contract test. Done when: the contract test passes against fallow 3.42.0, every v0 rule has bad and good fixtures, and the FP corpus shows each default-on rule at or below 1 finding per 1,000 lines.

v1, diffs and breadth. `--changed-since` and `--staged` (line-level filtering, `fallow audit`). Baseline save and compare. Go grammar. Rules marked v1 in section 4, including native complexity and exact duplication for non-JS languages and the two `pr/*` rules. `slopr rules --all` via `fallow schema`. `skill/SKILL.md` and pre-commit hook.

v2, optional and model-based. `dup/semantic` as an opt-in `--semantic` mode, clean-room from the slopo idea: tree-sitter units, embeddings from a configured OpenAI-compatible endpoint or local Ollama, cosine pairs, transitive clusters, distance boost, cluster hash ignore list; for JS/TS prefer `fallow similar-code`. Python `deps/undeclared-import`. Native `test/weakened`. Consider an MCP server (rmcp) only if agents need more than the JSON CLI. Consider a `cargo-fallow` adapter for Rust targets using the same subprocess pattern.

## 9. Risks

- Comment heuristics produce false positives and annoy users. Mitigation: FP corpus gate, `comment/obvious` off by default, every finding suppressible with a reason.
- fallow moves fast (five releases in ten days). Mitigation: rely only on semver-covered surfaces, pin a range, weekly contract test against latest.
- fallow missing or out of range on the machine (users install it separately, per the 2026-11-11 decision). Mitigation: startup check exits 3 with the found version, required range and install hint; `--fallow off` skips fallow explicitly.
- Double reporting between native rules and fallow. Mitigation: `via_fallow_for_js` flag in the rule table.
- tree-sitter grammar crates need a C compiler and lag tree-sitter core releases. Mitigation: phase 0 build check; keep the grammar list short.
- License contamination. Mitigation: no code from slopo (AGPL), vibecheck (FSL), slop-guard (no license) or aicomplex (no LICENSE file); phrase lists only from MIT or Apache-2.0 sources with provenance comments in `phrases.rs`.
- Phrase lists are English-centric. stop-ai-slop has ru, de, fr, es lists under MIT that can be ported later.

## 10. Licensing

| Source | License | What slopr may do |
|---|---|---|
| fallow | MIT | Run as a separate binary. No code copied, no linking. Credit in README. |
| aislop | MIT (Copyright 2025 heavykenny) | Port regex and phrase lists (for example `narrative-comments-patterns.ts`) with the MIT notice in `THIRD_PARTY_NOTICES.md`. |
| stop-ai-slop | MIT (Copyright 2026 WhiteBite) | Port phrase lists and Unicode ranges with the MIT notice. |
| sloplint, slopsieve, OpenPolicyKit, SynthScan, slopgate, ai-slop-cleaner, rayonx, AI-Slop-Linter-for-code, Slop-Popper, gretchenfitze/anti-slop, scanaislop/skills | MIT | Port with the MIT notice of each repo actually copied from. |
| agent-slop-lint | Apache-2.0 with NOTICE | Port with the Apache-2.0 license text, keep its NOTICE content, mark changed files. Metric formulas are from published research (cited in its `CITATIONS.md`). |
| axonscanner | Apache-2.0 | Same as above if anything is ported. |
| slopo | AGPL-3.0-or-later | Clean-room only. Do not copy code or file structure. The technique (embedding similarity, clustering, distance weighting) may be reimplemented from the README and the author's public benchmark article; pick thresholds from slopr's own measurement. |
| vibecheck | FSL-1.1-Apache-2.0 | Source-available, competing use restricted for two years per release. Ideas only, no code or rule text. |
| aicomplex | unverified (MIT claimed in package.json, no LICENSE file) | Ideas only until a LICENSE file exists. |
| slop-guard | none | All rights reserved. Ideas only. |

Rule names and detection ideas are not copyrightable, but copying regex lists is copying code, so every ported list carries its source and license in a comment.

## 11. Open questions

Resolved: separate fallow install, see decision 5.1.1 (2026-11-11).

1. Which languages matter most beyond TS/JS? The plan picks Python and Rust for v0, Go for v1.
2. Final name. `slopr` is a placeholder; `slop`, `aislop`, `sloplint`, `slopgate`, `slop-guard` are taken by sources above.
3. Should `comment/untracked-todo` default to info or off? Sources disagree (aislop info, stop-ai-slop warning).
