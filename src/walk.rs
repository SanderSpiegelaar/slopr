use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use rayon::prelude::*;

use crate::source::{Lang, SourceFile};

const MAX_FILE_BYTES: u64 = 1 << 20;
const BINARY_SNIFF_BYTES: usize = 8 << 10;

const TEST_GLOBS: &[&str] = &[
    "**/test/**",
    "**/tests/**",
    "**/__tests__/**",
    "**/*.test.*",
    "**/*.spec.*",
    "**/test_*.py",
    "**/*_test.py",
    "**/conftest.py",
    "**/*_test.go",
];

/// Walks `paths` under the current directory, the scan root, and loads every file.
///
/// # Errors
///
/// A path that does not exist or lies outside the root.
pub fn files(paths: &[PathBuf], exclude: &GlobSet) -> Result<Vec<SourceFile>> {
    let root = std::env::current_dir()?.canonicalize()?;
    let test_globs = test_globs();
    let mut found = Vec::new();
    for path in paths {
        let full = path
            .canonicalize()
            .with_context(|| format!("{}", path.display()))?;
        let Ok(relative) = full.strip_prefix(&root) else {
            bail!(
                "{} is outside the scan root {}",
                path.display(),
                root.display()
            );
        };
        let start = if relative.as_os_str().is_empty() {
            Path::new(".")
        } else {
            relative
        };
        for entry in WalkBuilder::new(start).require_git(false).build() {
            match entry {
                Ok(entry) if entry.file_type().is_some_and(|t| t.is_file()) => {
                    found.push(entry.into_path());
                }
                Ok(_) => {}
                Err(err) => eprintln!("slopr: skipped {err}"),
            }
        }
    }
    found.sort_unstable();
    found.dedup();

    let loaded: Vec<_> = found
        .par_iter()
        .map(|path| load(path, exclude, &test_globs))
        .collect();
    let mut files = Vec::with_capacity(loaded.len());
    for result in loaded {
        match result {
            Ok(Some(file)) => files.push(file),
            Ok(None) => {}
            Err(err) => eprintln!("slopr: skipped {err:#}"),
        }
    }
    Ok(files)
}

/// `None` for files skipped silently: excluded, over 1 MiB, binary, or not UTF-8.
fn load(path: &Path, exclude: &GlobSet, test_globs: &GlobSet) -> Result<Option<SourceFile>> {
    let shown = path.display();
    let Some(relative) = root_relative(path) else {
        bail!("{shown}: path is not UTF-8");
    };
    if exclude.is_match(&relative) {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    let file = fs::File::open(path).with_context(|| format!("{shown}"))?;
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("{shown}"))?;
    if bytes.len() as u64 > MAX_FILE_BYTES
        || bytes[..bytes.len().min(BINARY_SNIFF_BYTES)].contains(&0)
    {
        return Ok(None);
    }
    let Ok(text) = String::from_utf8(bytes) else {
        return Ok(None);
    };
    let lang = lang_for(&relative);
    let is_test = test_globs.is_match(&relative);
    Ok(Some(SourceFile::new(relative, text, lang, is_test)))
}

/// Forward-slash path with the leading `./` dropped.
fn root_relative(path: &Path) -> Option<String> {
    let path = path.strip_prefix(".").unwrap_or(path);
    let parts: Option<Vec<&str>> = path.components().map(|c| c.as_os_str().to_str()).collect();
    parts.map(|parts| parts.join("/"))
}

fn lang_for(path: &str) -> Option<Lang> {
    match Path::new(path).extension()?.to_str()? {
        "ts" | "mts" | "cts" => Some(Lang::TypeScript),
        "tsx" => Some(Lang::Tsx),
        "js" | "mjs" | "cjs" | "jsx" => Some(Lang::JavaScript),
        "py" | "pyi" => Some(Lang::Python),
        "rs" => Some(Lang::Rust),
        _ => None,
    }
}

fn test_globs() -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for glob in TEST_GLOBS {
        builder.add(Glob::new(glob).expect("test globs are valid"));
    }
    builder.build().expect("test globs are valid")
}
