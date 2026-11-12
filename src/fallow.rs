use std::path::{Path, PathBuf};
use std::process::Command;

use ignore::WalkBuilder;
use semver::{Version, VersionReq};

use crate::config::{Fallow, FallowMode};

/// fallow is required but missing or outside the configured range.
#[derive(Debug, thiserror::Error)]
#[error(
    "fallow {} does not satisfy {required}. Install it with: npm install -D fallow@^3.42 (or cargo install fallow-cli --version ^3.42). Use --fallow off to skip.",
    found.as_deref().unwrap_or("not found")
)]
pub struct Unsatisfied {
    found: Option<String>,
    required: VersionReq,
}

/// Ok when fallow is off, not required, or found in range.
///
/// # Errors
///
/// fallow is required and missing or out of range.
pub fn check(config: &Fallow, paths: &[PathBuf]) -> Result<(), Unsatisfied> {
    let required = match config.enabled {
        FallowMode::Off => false,
        FallowMode::On => true,
        FallowMode::Auto => has_package_json(paths),
    };
    if !required {
        return Ok(());
    }
    let local = Path::new("node_modules/.bin/fallow");
    let bin = match &config.bin {
        Some(bin) => bin.as_path(),
        None if local.is_file() => local,
        None => Path::new("fallow"),
    };
    let found = Command::new(bin).arg("--version").output().ok().map(|out| {
        let text = String::from_utf8_lossy(&out.stdout);
        let text = text.trim();
        text.strip_prefix("fallow ").unwrap_or(text).to_owned()
    });
    if let Some(found) = &found
        && Version::parse(found).is_ok_and(|v| config.version.matches(&v))
    {
        return Ok(());
    }
    Err(Unsatisfied {
        found,
        required: config.version.clone(),
    })
}

fn has_package_json(paths: &[PathBuf]) -> bool {
    Path::new("package.json").is_file()
        || paths.iter().any(|path| {
            WalkBuilder::new(path)
                .require_git(false)
                .build()
                .flatten()
                .any(|entry| {
                    entry.file_name() == "package.json"
                        && entry.file_type().is_some_and(|t| t.is_file())
                })
        })
}
