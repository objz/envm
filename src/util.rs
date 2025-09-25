use std::process::Command;

use color_eyre::eyre::{Result, eyre};

use crate::logging as log;

pub fn run_cmd(cmd: &mut Command) -> Result<()> {
    let display = format!("{:?}", cmd);
    let status = cmd.status()?;
    if !status.success() {
        log::error(format!(
            "command failed: {} ({:?})",
            display,
            status.code()
        ));
        return Err(eyre!("command failed: {}", display));
    }
    Ok(())
}

pub fn is_root() -> bool {
    nix::unistd::Uid::effective().is_root()
}
