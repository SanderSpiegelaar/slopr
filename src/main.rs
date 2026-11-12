use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::builder::PossibleValue;
use clap::{Parser, Subcommand, ValueEnum};

use crate::finding::{Finding, Severity};

mod config;
mod fallow;
mod finding;
mod report;
mod rules;
mod source;
mod suppress;
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
        #[arg(long, value_enum, value_name = "MODE")]
        fallow: Option<FallowFlag>,
        #[arg(long, value_name = "PATH")]
        fallow_bin: Option<PathBuf>,
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

#[derive(Clone, Copy, ValueEnum)]
enum FallowFlag {
    Auto,
    On,
    Off,
}

impl From<FallowFlag> for config::FallowMode {
    fn from(flag: FallowFlag) -> Self {
        match flag {
            FallowFlag::Auto => Self::Auto,
            FallowFlag::On => Self::On,
            FallowFlag::Off => Self::Off,
        }
    }
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
            fallow,
            fallow_bin,
        } => {
            let mut config = config::load()?;
            if let Some(mode) = fallow {
                config.fallow.enabled = mode.into();
            }
            if fallow_bin.is_some() {
                config.fallow.bin = fallow_bin;
            }
            if let Err(err) = fallow::check(&config.fallow, &paths) {
                eprintln!("slopr: {err}");
                return Ok(ExitCode::from(3));
            }
            check(&config, &paths, format, max_severity_exit)
        }
        Command::Rules => {
            io::stdout().write_all(report::render_rules().as_bytes())?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Init => config::init().map(|()| ExitCode::SUCCESS),
    }
}

fn check(
    config: &config::Config,
    paths: &[PathBuf],
    format: Format,
    gate: Severity,
) -> anyhow::Result<ExitCode> {
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
    suppress::apply(&files, &mut findings);
    findings.sort_by(|a, b| report_order(a).cmp(&report_order(b)));
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

fn report_order(f: &Finding) -> (&[u8], u32, u32, &str, &str) {
    (
        f.path.as_bytes(),
        f.span.start_line,
        f.span.start_col,
        &f.rule,
        &f.message,
    )
}
