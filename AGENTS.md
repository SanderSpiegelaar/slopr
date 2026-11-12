# AGENTS.md

slopr is a Rust CLI that flags the code-quality slop AI coding agents leave behind. It runs native tree-sitter rules and passes through fallow for JS/TS projects.

Status: planning. `src/main.rs` is a stub and nothing in the plan is built yet. The next step is the Phase 0 spike.

## Start here

Open [docs/index.md](docs/index.md). Its task router lists the files your task needs. Read only those.

## Commands

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

The fallow contract test runs only when `FALLOW_BIN` points at a fallow binary.

## Hard constraints

- One crate. Use modules, not workspace crates. See [architecture](docs/reference/architecture.md).
- `slopr check` makes no network calls and sends no telemetry.
- fallow is a subprocess only. Never depend on fallow's Rust crates or its NAPI package. See [decisions](docs/research/decisions.md).
- Copy no code or pattern lists from slopo, vibecheck, slop-guard or aicomplex. Every ported list cites its source and license. See [licensing](docs/reference/licensing.md).
- Add only crates listed in [dependencies](docs/reference/dependencies.md). Any other crate needs the user's approval.

## Skills

Project skills live in `.agents/skills/`, and `.claude/skills/` links to them.

- `rust-best-practices` when writing or changing Rust.
- `rust-review` on your diff before handing back.
- `rust-port` when porting a detector from a TypeScript or Python source repo.

## Keeping the plan right

- Each fact lives in one file under `docs/`. Change it there and nowhere else.
- A new or reversed decision gets a record in [decisions](docs/research/decisions.md), plus an update to the reference file it changes.
- A new kind of task or a new doc file gets a row in the index task router.
