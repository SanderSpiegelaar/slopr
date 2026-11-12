use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::builder::PossibleValue;
use clap::{Parser, Subcommand, ValueEnum};

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
        #[arg(long, value_enum, default_value_t = Format::Human)]
        format: Format,
        #[arg(long, value_enum, value_name = "LEVEL", default_value_t = Severity::Error)]
        max_severity_exit: Severity,
    },
    /// List native rules
    Rules,
    /// Write slopr.toml with defaults
    Init,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Human,
    Json,
    Sarif,
}

impl ValueEnum for Severity {
    fn value_variants<'a>() -> &'a [Self] {
        &[Severity::Info, Severity::Warn, Severity::Error]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(match self {
            Severity::Info => "info",
            Severity::Warn => "warn",
            Severity::Error => "error",
        }))
    }
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
        Command::Check {
            paths,
            format,
            max_severity_exit,
        } => check(&paths, format, max_severity_exit),
        Command::Rules => {
            io::stdout().write_all(report::render_rules().as_bytes())?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Init => config::init().map(|()| ExitCode::SUCCESS),
    }
}

fn check(paths: &[PathBuf], format: Format, gate: Severity) -> anyhow::Result<ExitCode> {
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
    let out = match format {
        Format::Human => report::render_human(&findings),
        Format::Json => report::render_json(&findings),
        Format::Sarif => report::render_sarif(&findings),
    };
    io::stdout().write_all(out.as_bytes())?;
    let gate_failed = findings.iter().any(|f| f.severity >= gate);
    Ok(if gate_failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
