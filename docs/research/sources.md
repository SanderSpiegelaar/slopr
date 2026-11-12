# Sources

## Collection

- `rafal-qa/slopo`: 1 repo.
- `github.com/scanaislop` org: 7 public repos (API listing on 2026-11-11).
- `github.com/topics/ai-slop`: 546 repos returned by the GitHub search API (`q=topic:ai-slop`) on 2026-11-11. Most are prose humanizers, UI design skills, browser extensions and games. Filtered on description keywords for code quality, lint, static analysis, dead code, duplication, comments, PR review, then dropped no-code skill packs and unrelated projects.
- fallow: `fallow-rs/fallow` plus `https://fallow.tools/llms-full.txt` (docs source commit `256a68ca`).

Totals: 549 unique candidate repos (1 slopo, 7 org, 546 topic; 5 org repos also carry the topic). 19 analyzed as rule sources, plus fallow as the integration target, 20 in total. 530 dropped: 525 topic repos (prose humanizers, UI design skills, browser extensions, games, no detector code) and 5 scanaislop org repos: `flashwave` (unrelated Expo strobe-light app, no license), `aislop-py` (pipx wrapper of the same CLI), `homebrew-tap` (packaging), `mintlify-docs` (docs only), `.github` (profile). Code was read from shallow clones in `/workspace`, which were deleted after analysis.

## Repos analyzed, ranked by relevance then maintenance

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

## Per-repo notes (code read, paths cited)

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
