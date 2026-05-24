use std::collections::HashMap;
use std::{env, fs, path::PathBuf};

use serde::Deserialize;
use vis_diagnostics::Severity;

#[derive(Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    components: HashMap<String, String>,
    #[serde(default)]
    rules: HashMap<String, String>,
    #[serde(default)]
    exclude: Vec<String>,
}

#[derive(Default, Debug)]
#[allow(dead_code)]
pub struct Config {
    pub components: HashMap<String, String>,
    pub rules: HashMap<String, Severity>,
    pub exclude: Vec<String>,
}

impl Config {
    pub fn load() -> Option<Self> {
        let path = find_config()?;
        let contents = fs::read_to_string(&path).ok()?;
        let raw: RawConfig = toml::from_str(&contents).ok()?;

        let components: HashMap<String, String> = raw
            .components
            .into_iter()
            .map(|(k, v)| (k.to_ascii_lowercase(), v))
            .collect();

        let rules: HashMap<String, Severity> = raw
            .rules
            .into_iter()
            .map(|(k, v)| {
                let severity = match v.as_str() {
                    "warn" | "warning" => Severity::Warning,
                    "info" => Severity::Info,
                    "off" => Severity::Off,
                    _ => Severity::Error,
                };
                (k, severity)
            })
            .collect();

        Some(Config {
            components,
            rules,
            exclude: raw.exclude,
        })
    }
}

fn find_config() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    for ancestor in cwd.ancestors() {
        let candidate = ancestor.join(".visrc.toml");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}
