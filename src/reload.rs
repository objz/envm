use std::process::{Command, Stdio};

use color_eyre::eyre::Result;

use crate::logging as log;
use crate::scope::Scope;
use crate::util::{is_root, run_cmd};

pub fn reload(scope: &Scope, changed_keys: &[&str]) -> Result<()> {
    match scope {
        Scope::User => {
            run_cmd(Command::new("systemctl").args(["--user", "daemon-reload"]))?;

            if !changed_keys.is_empty() {
                let mut cmd = Command::new("systemctl");
                cmd.args(["--user", "import-environment"]);
                for k in changed_keys {
                    cmd.arg(k);
                }
                let _ = cmd.stdout(Stdio::null()).stderr(Stdio::null()).status();
            }
        }
        Scope::System => {
            if !is_root() {
                log::error(format!(
                    "Writing to /etc/environment.d requires root. Re-run with sudo or as root"
                ));
            }
            run_cmd(Command::new("systemctl").arg("daemon-reload"))?;
        }
    }
    Ok(())
}
