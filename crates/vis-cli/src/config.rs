use std::collections::HashMap;
use std::{env, fs, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    components: HashMap<String, String>,
}

#[derive(Default, Debug)]
pub struct Config {
    pub components: HashMap<String, String>,
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
        Some(Config { components })
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
