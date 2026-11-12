# Config: `slopr.toml`

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
