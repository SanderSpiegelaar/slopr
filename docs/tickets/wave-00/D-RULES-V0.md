---
id: D-RULES-V0
title: Decide language coverage and node patterns for v0 rules
kind: decision
depends_on: []
docs: [docs/reference/rules.md#merged-rule-catalog, docs/reference/testing.md#test-strategy]
touches: [docs/reference/rules.md]
creates: [docs/reference/rules.md#v0-language-coverage]
covers: [gap:D08a, gap:D24]
verify: grep -qx '## v0 language coverage' docs/reference/rules.md && grep -q 'U+FEFF at byte offset 0' docs/reference/rules.md
---

## Goal

Say, for each v0 rule, which languages it runs on and which syntax it matches, so rule tickets build exactly that.

## Scope

Add `## v0 language coverage` after the catalog. Use TS for TypeScript and Tsx, JS for JavaScript (parsed with the TSX grammar), PY and RS. "Comment" means a comment node from `SourceFile::comments()`. It skips doc comments (`///`, `//!`, `/** */`) and Python docstrings, and gives the text with its markers stripped. "String" means a string node from `SourceFile::strings()`.

| Rule | Langs | Matches |
|---|---|---|
| `comment/narrative` | TS JS PY RS | Comment whose text starts (case-insensitive, after whitespace) with a phrase from the ported narrative list. |
| `comment/section-divider` | TS JS PY RS | Comment matching `^([=\-*#~_])\1{3,}` or `^[=\-*#~_]{3,}\s+.+?\s+[=\-*#~_]{3,}$`. |
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

## Out of scope

- v1 rules, which belong to `D-RULES-V1`.
- Go, which belongs to `D-RULES-V1`.

## Acceptance criteria

- The section and table exist with the content above.
- `python3 scripts/check-tickets.py` passes.
