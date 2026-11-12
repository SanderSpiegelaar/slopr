# CLI

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

## Exit gate

`--max-severity-exit` takes `info`, `warn` or `error`. The default is `error`.

- 0: no reported finding is at or above the gate level.
- 1: the gate failed. A reported finding is one left after suppressions, baseline and diff filtering. The gate fails when any reported finding is at or above the level.
- 2: usage or config error. It happens before any scan. clap usage errors already exit 2.
- 3: fallow is required but missing, out of range, or failed. slopr still prints the findings, then exits 3 whatever the gate says.

A file that cannot be read prints `slopr: skipped <path>: <reason>` to stderr. It does not change the exit code.

## Output formats

Findings are sorted by path (byte order), then start_line, start_col, rule and message. No format contains an absolute path, so snapshots stay stable.

### human

Output goes to stdout. Each finding is one line, `<path>:<line>:<col> <severity> <rule> <message>`, with single spaces. A blank line separates files. After the findings come a blank line, one `<count> <rule>` line per rule in rule order, and a last line `<n> findings (<e> error, <w> warn, <i> info)`. With no findings the whole output is `0 findings`. There is no color.

### json

```
{"schema_version": 1, "version": "<crate version>", "findings": [...], "summary": {"total": n, "by_severity": {"error": e, "warn": w, "info": i}, "by_rule": {"<rule>": n}}, "tools": [{"name": "fallow", "version": "<x.y.z>" or null, "status": "ok" | "skipped" | "missing" | "failed"}]}
```

Each finding is the `Finding` struct. `span` is an object. `origin` is `{"kind": "native"}` or `{"kind": "fallow", "version", "command"}`. `by_rule` is sorted by key. The output is pretty-printed with 2 spaces and ends with a newline. The `fallow` entry in `tools` is always present. Its status is `skipped` when fallow is off or not required. Bump `schema_version` on any rename or removal, following fallow's policy.

### sarif

```
{"$schema": "https://json.schemastore.org/sarif-2.1.0.json", "version": "2.1.0", "runs": [...]}
```

The first run is slopr's. fallow's runs are appended after it.

- `tool.driver` has `name: "slopr"`, `version` and `rules`. `rules` lists every native rule plus `suppress/invalid-directive`, sorted by id. Each rule has `id`, `shortDescription.text` (its summary) and `defaultConfiguration.level`.
- Each result has `ruleId`, `level`, `message.text`, one location and `partialFingerprints: {"slopr/v1": fingerprint}`. `level` maps error to `error`, warn to `warning` and info to `note`.
- The location has `artifactLocation.uri` (the path), `uriBaseId: "%SRCROOT%"`, and a `region` with all four span fields.
- fallow findings appear only in fallow's appended runs, never in slopr's run.

## CI and coding-agent integration

- GitHub Actions: `slopr check --format sarif > slopr.sarif` then `github/codeql-action/upload-sarif`. Gate step: `slopr check --changed-since origin/${{ github.base_ref }}`.
- pre-commit: `.pre-commit-hooks.yaml` entry running `slopr check --staged`.
- Agents: `skill/SKILL.md`, printed by `slopr skill`. It tells the agent to run `slopr check --changed-since HEAD --format json` before handing back, fix error findings, never add a suppression without a reason, and treat `dup/semantic` clusters as candidates to verify (the slopo skills pattern, `src/slopo/agent/configs/*`). No MCP server in v0 or v1; agents can use fallow's MCP for JS/TS depth.
