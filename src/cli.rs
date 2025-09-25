use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};
use color_eyre::eyre::Result;

use crate::env::{EnvMap, load_env_map, save_env_map, validate_key};
use crate::logging as log;
use crate::reload::reload;
use crate::scope::Scope;
use std::io::{self, Write};
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Manage global environment variables (systemd environment.d)"
)]
struct Cli {
    /// Use system scope (/etc/environment.d) instead of user scope (~/.config/environment.d)
    #[arg(long, action = ArgAction::SetTrue)]
    system: bool,

    /// Path of the managed environment file (defaults: 90-genv.conf in the appropriate environment.d)
    #[arg(long)]
    file: Option<PathBuf>,

    #[command(subcommand)]
    cmd: CommandKind,
}

#[derive(Subcommand, Debug)]
enum CommandKind {
    /// Add a new variable 
    Add { key: String, value: String },

    /// Edit an existing variable
    Edit { key: String, value: String },

    /// Remove a variable
    Remove { key: String },

    /// List variables managed by this file
    List,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let scope = if cli.system {
        Scope::System
    } else {
        Scope::User
    };

    let env_file = crate::env::default_env_file(scope, cli.file.as_deref())?;

    let mut map: EnvMap = match load_env_map(&env_file) {
        Ok(m) => m,
        Err(err) if err.to_string().contains("No such file") => EnvMap::default(),
        Err(err) if err.to_string().contains("not found") => EnvMap::default(),
        Err(err) if err.to_string().contains("open") => EnvMap::default(),
        Err(e) => return Err(e),
    };

    match &cli.cmd {
        CommandKind::Add { key, value } => {
            validate_key(key)?;
            if let Some(existing) = map.get(key) {
                if existing == value {
                    log::warn(format!("variable {} already set to the same value", key));
                    return Ok(());
                }
                print!(
                    "variable {} already exists with value '{}'. Overwrite? [y/N]: ",
                    key, existing
                );
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
                    log::warn(format!("aborted adding variable {}", key));
                    return Ok(());
                }
            }
            map.insert(key.clone(), value.clone());
            save_env_map(&env_file, &map)?;
            reload(&scope, &[key.as_str()])?;
            log::success(format!("added variable {}={}", key, value));
        }

        CommandKind::Edit { key, value } => {
            validate_key(key)?;
            let Some(existing) = map.get(key) else {
                log::error(format!("key {} does not exist", key));
                return Ok(()); 
            };

            if existing == value {
                log::warn(format!("variable {} already has that value", key));
                return Ok(());
            }
            map.insert(key.clone(), value.clone());
            save_env_map(&env_file, &map)?;
            reload(&scope, &[key.as_str()])?;
            log::success(format!("edited variable {}={}", key, value));
        }

        CommandKind::Remove { key } => {
            validate_key(key)?;
            if map.remove(key).is_some() {
                save_env_map(&env_file, &map)?;
                reload(&scope, &[] as &[&str])?;
                log::success(format!("removed variable {}", key));
            } else {
                log::warn(format!("key {} is not present", key));
            }
        }

        CommandKind::List => {
            let mut items: Vec<(String, String)> =
                map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            items.sort_by(|a, b| a.0.cmp(&b.0));
            log::kv_listing(&env_file.display().to_string(), &items);
        }
    }

    Ok(())
}
