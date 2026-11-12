use std::collections::HashMap;
use std::fmt::Write;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{fs, io};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use semver::VersionReq;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::finding::Severity;
use crate::rules::{FAMILIES, Rule};

pub const FILE: &str = "slopr.toml";

const DEFAULT_ISSUE_PATTERN: &str = r"(#\d+|[A-Z]+-\d+|https?://)";

/// The parsed `slopr.toml`.
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default, deserialize_with = "glob_set")]
    pub exclude: GlobSet,
    #[serde(default)]
    rules: Rules,
    #[serde(default)]
    overrides: Vec<Override>,
    #[serde(default)]
    #[allow(dead_code, reason = "read by F01 and F02")]
    pub fallow: Fallow,
}

#[derive(Default, Deserialize)]
struct Rules {
    #[serde(default)]
    settings: RuleSettings,
    #[serde(flatten)]
    levels: HashMap<RuleId, Level>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Override {
    #[serde(deserialize_with = "glob_set")]
    paths: GlobSet,
    #[serde(default)]
    rules: HashMap<RuleId, Level>,
}

/// The ID of a native rule.
#[derive(PartialEq, Eq, Hash)]
struct RuleId(&'static str);

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Level {
    Off,
    Info,
    Warn,
    Error,
}

#[derive(Deserialize)]
#[serde(from = "SettingsTables")]
pub struct RuleSettings {
    pub todo_issue_pattern: Regex,
}

/// `[rules.settings]`: one table per rule, holding only the keys built so far.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsTables {
    #[serde(rename = "comment/untracked-todo", default)]
    untracked_todo: UntrackedTodo,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct UntrackedTodo {
    #[serde(default, deserialize_with = "regex")]
    issue_pattern: Option<Regex>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Fallow {
    pub enabled: FallowMode,
    /// `None` when unset, written as `""`.
    #[serde(deserialize_with = "bin", serialize_with = "ser_bin")]
    pub bin: Option<PathBuf>,
    #[serde(deserialize_with = "version", serialize_with = "ser_version")]
    pub version: VersionReq,
    pub commands: Vec<FallowCommand>,
    pub args: Vec<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FallowMode {
    Auto,
    On,
    Off,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FallowCommand {
    DeadCode,
    Dupes,
    Health,
    Security,
    Architecture,
    Flags,
}

impl Config {
    /// The severity of `rule` on root-relative `path`, or `None` when the rule is off.
    pub fn severity(&self, rule: &Rule, path: &str) -> Option<Severity> {
        let id = RuleId(rule.id);
        let mut severity = match self.rules.levels.get(&id) {
            Some(level) => level.severity(),
            None => rule.default_on.then_some(rule.default_severity),
        };
        for entry in self.overrides.iter().filter(|o| o.paths.is_match(path)) {
            if let Some(level) = entry.rules.get(&id) {
                severity = level.severity();
            }
        }
        severity
    }

    pub fn settings(&self) -> &RuleSettings {
        &self.rules.settings
    }
}

impl Level {
    fn severity(self) -> Option<Severity> {
        match self {
            Level::Off => None,
            Level::Info => Some(Severity::Info),
            Level::Warn => Some(Severity::Warn),
            Level::Error => Some(Severity::Error),
        }
    }
}

impl<'de> Deserialize<'de> for RuleId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let id = String::deserialize(d)?;
        native_rule(&id)
            .map(|rule| RuleId(rule.id))
            .ok_or_else(|| D::Error::custom(format!("unknown rule `{id}`")))
    }
}

impl Default for RuleSettings {
    fn default() -> Self {
        RuleSettings {
            todo_issue_pattern: Regex::new(DEFAULT_ISSUE_PATTERN)
                .expect("default issue pattern is a valid regex"),
        }
    }
}

impl From<SettingsTables> for RuleSettings {
    fn from(tables: SettingsTables) -> Self {
        let mut settings = RuleSettings::default();
        if let Some(pattern) = tables.untracked_todo.issue_pattern {
            settings.todo_issue_pattern = pattern;
        }
        settings
    }
}

impl Default for Fallow {
    fn default() -> Self {
        Fallow {
            enabled: FallowMode::Auto,
            bin: None,
            version: VersionReq::parse(">=3.42, <4").expect("default version is valid"),
            commands: vec![
                FallowCommand::DeadCode,
                FallowCommand::Dupes,
                FallowCommand::Health,
            ],
            args: Vec::new(),
        }
    }
}

pub fn native_rule(id: &str) -> Option<&'static Rule> {
    FAMILIES
        .iter()
        .copied()
        .flatten()
        .find(|rule| rule.id == id)
}

/// Reads `./slopr.toml`, or the defaults when there is none.
///
/// # Errors
///
/// The file cannot be read or is not a valid config.
pub fn load() -> Result<Config> {
    match fs::read_to_string(FILE) {
        Ok(text) => toml::from_str(&text).context(FILE),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(Config::default()),
        Err(err) => Err(err).context(FILE),
    }
}

/// Writes `./slopr.toml` holding every default.
///
/// # Errors
///
/// The file already exists or cannot be written.
pub fn init() -> Result<()> {
    fs::File::create_new(FILE)
        .and_then(|mut file| io::Write::write_all(&mut file, starter().as_bytes()))
        .context(FILE)
}

/// The config.md example shape, with every native rule and setting at its default.
fn starter() -> String {
    let mut out = String::from("exclude = []\n\n[rules]\n");
    for rule in FAMILIES.iter().copied().flatten() {
        let level = if rule.default_on {
            rule.default_severity.to_string()
        } else {
            "off".to_string()
        };
        writeln!(out, "\"{}\" = \"{level}\"", rule.id).expect("writing to a String");
    }
    write!(
        out,
        "\n[rules.settings.\"comment/untracked-todo\"]\nissue_pattern = '{DEFAULT_ISSUE_PATTERN}'\n\n[fallow]\n{}",
        toml::to_string(&Fallow::default()).expect("fallow defaults serialize")
    )
    .expect("writing to a String");
    out
}

fn glob_set<'de, D: Deserializer<'de>>(d: D) -> Result<GlobSet, D::Error> {
    let mut builder = GlobSetBuilder::new();
    for glob in Vec::<String>::deserialize(d)? {
        builder.add(Glob::new(&glob).map_err(D::Error::custom)?);
    }
    builder.build().map_err(D::Error::custom)
}

fn regex<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Regex>, D::Error> {
    Regex::new(&String::deserialize(d)?)
        .map(Some)
        .map_err(D::Error::custom)
}

fn bin<'de, D: Deserializer<'de>>(d: D) -> Result<Option<PathBuf>, D::Error> {
    let bin = String::deserialize(d)?;
    Ok((!bin.is_empty()).then(|| bin.into()))
}

fn version<'de, D: Deserializer<'de>>(d: D) -> Result<VersionReq, D::Error> {
    VersionReq::parse(&String::deserialize(d)?).map_err(D::Error::custom)
}

fn ser_bin<S: Serializer>(bin: &Option<PathBuf>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(bin.as_deref().and_then(|p| p.to_str()).unwrap_or(""))
}

fn ser_version<S: Serializer>(version: &VersionReq, s: S) -> Result<S::Ok, S::Error> {
    s.collect_str(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starter_parses_back_to_defaults() {
        let config: Config = toml::from_str(&starter()).expect("starter parses");
        let defaults = Config::default();
        assert!(config.exclude.is_empty());
        assert!(config.overrides.is_empty());
        assert_eq!(config.fallow, defaults.fallow);
        assert_eq!(
            config.settings().todo_issue_pattern.as_str(),
            defaults.settings().todo_issue_pattern.as_str()
        );
        for rule in FAMILIES.iter().copied().flatten() {
            assert_eq!(
                config.severity(rule, "a.ts"),
                defaults.severity(rule, "a.ts")
            );
        }
    }
}
