use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::config::RuleSettings;
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
    let Command::Check { paths } = cli.command;
    let files = walk::files(&paths)?;
    let findings = rules::run_rules(&files, &RuleSettings::default());
    io::stdout().write_all(report::render_human(&findings).as_bytes())?;
    let gate_failed = findings.iter().any(|f| f.severity >= Severity::Error);
    Ok(if gate_failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
