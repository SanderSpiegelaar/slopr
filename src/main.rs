use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::finding::Severity;

mod config;
mod finding;
mod report;
mod rules;
mod source;
mod walk;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check files for slop
    Check {
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
    },
    /// Write slopr.toml with defaults
    Init,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("slopr: {err:#}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<ExitCode> {
    match cli.command {
        Command::Check { paths } => check(&paths),
        Command::Init => config::init().map(|()| ExitCode::SUCCESS),
    }
}

fn check(paths: &[PathBuf]) -> anyhow::Result<ExitCode> {
    let config = config::load()?;
    let files = walk::files(paths, &config.exclude)?;
    let mut findings = rules::run_rules(&files, config.settings());
    findings.retain_mut(|finding| {
        let rule = config::native_rule(&finding.rule).expect("findings come from native rules");
        let Some(severity) = config.severity(rule, &finding.path) else {
            return false;
        };
        finding.severity = severity;
        true
    });
    io::stdout().write_all(report::render_human(&findings).as_bytes())?;
    let gate_failed = findings.iter().any(|f| f.severity >= Severity::Error);
    Ok(if gate_failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
