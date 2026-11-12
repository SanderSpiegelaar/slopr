# fallow integration

## Behavior

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
