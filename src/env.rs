use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use color_eyre::eyre::{Result};

use crate::logging as log;
use crate::scope::Scope;

pub type EnvMap = BTreeMap<String, String>;

pub fn default_env_file(scope: Scope, custom: Option<&Path>) -> Result<PathBuf> {
    if let Some(c) = custom {
        return Ok(c.to_path_buf());
    }

    let base = match scope {
        Scope::User => {
            let mut base = home::home_dir().unwrap_or_else(|| PathBuf::from("~"));
            base.push(".config/environment.d");
            base
        }
        Scope::System => PathBuf::from("/etc/environment.d"),
    };

    Ok(base.join("90-genv.conf"))
}

pub fn load_env_map(path: &Path) -> Result<EnvMap> {
    let mut map = EnvMap::new();

    if !path.exists() {
        return Ok(map);
    }

    let f = fs::File::open(path)?;
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        } else {
            log::warn(format!(
                "skipping malformed line in {}: {}",
                path.display(),
                line
            ));
        }
    }

    Ok(map)
}

pub fn save_env_map(path: &Path, map: &EnvMap) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut f = fs::File::create(path)?;
    writeln!(f, "# Managed by genv")?;
    for (k, v) in map {
        writeln!(f, "{}={}", k, v)?;
    }

    Ok(())
}

pub fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        log::error("variable name cannot be empty");
    }
    if key.chars().any(|c| c.is_whitespace() || c == '=') {
        log::error(format!("invalid variable name: {}", key));
    }
    Ok(())
}
