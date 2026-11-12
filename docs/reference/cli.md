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

Exit codes: 0 no finding at or above the gate severity after suppressions and baseline; 1 gate failed; 2 usage or config error; 3 required external tool missing or failed.

## Output

- human: grouped by file, `path:line:col severity rule message`, summary counts by rule.
- json: `{ "schema_version": 1, "version", "findings": [Finding], "summary": {...}, "tools": [{"name": "fallow", "version", "status"}] }`. Bump `schema_version` on any rename or removal, following fallow's policy.
- sarif: SARIF 2.1.0 with one run for slopr (tool.driver.rules from `RULES`, `partialFingerprints` from `fingerprint`) plus fallow's runs appended.

## CI and coding-agent integration

- GitHub Actions: `slopr check --format sarif > slopr.sarif` then `github/codeql-action/upload-sarif`. Gate step: `slopr check --changed-since origin/${{ github.base_ref }}`.
- pre-commit: `.pre-commit-hooks.yaml` entry running `slopr check --staged`.
- Agents: `skill/SKILL.md`, printed by `slopr skill`. It tells the agent to run `slopr check --changed-since HEAD --format json` before handing back, fix error findings, never add a suppression without a reason, and treat `dup/semantic` clusters as candidates to verify (the slopo skills pattern, `src/slopo/agent/configs/*`). No MCP server in v0 or v1; agents can use fallow's MCP for JS/TS depth.
