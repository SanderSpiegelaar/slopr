# Dependencies (versions from crates.io on 2026-11-11)

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

Phase 0 must verify that the four grammar crates build against tree-sitter 0.27 through the `tree-sitter-language` `LanguageFn` interface (`parser.set_language(&tree_sitter_rust::LANGUAGE.into())`).
