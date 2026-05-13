use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};

const SYSTEMD_SERVICE: &str = include_str!("../assets/bot.service");
const SYSTEMD_SERVICE_PATH: &str = "/etc/systemd/system/app-pulse-bot.service";
const SYSTEMD_SERVICE_NAME: &str = "app-pulse-bot.service";
const BIN_PATH: &str = "/usr/local/bin/app-pulse-bot";

pub fn install(force: bool) -> Result<()> {
    let service_path = PathBuf::from(SYSTEMD_SERVICE_PATH);
    let service_path_existed = service_path.exists();
    if !force && service_path_existed {
        eprintln!("service already installed, use --force to overwrite");
    }

    if force || !service_path_existed {
        ensure_dir(&service_path).context("creating systemd dir")?;
        std::fs::write(service_path, SYSTEMD_SERVICE).context("failed to write systemd unit")?;

        if service_path_existed {
            systemctl(&["daemon-reload"]).context("reloading systemd daemon")?;
        }
    }

    install_bin()?;

    if service_path_existed {
        restart()?;
    } else {
        systemctl_service(&["enable", "--now"]).context("enabling")?;
    }

    eprintln!("done");

    Ok(())
}

pub fn restart() -> Result<()> {
    systemctl_service(&["restart"]).context("restarting")
}

fn install_bin() -> Result<()> {
    let bin_path = PathBuf::from(BIN_PATH);
    ensure_dir(&bin_path).context("creating bin dir")?;

    let current_exe = std::env::current_exe().context("failed to get current exe")?;
    if current_exe == bin_path {
        bail!("you're running binary which is already installed");
    }
    std::fs::rename(current_exe, bin_path).context("failed to move current exe to target dir")?;
    Ok(())
}

fn ensure_dir(file_path: &Path) -> Result<()> {
    if let Some(dir) = file_path.parent()
        && !dir.exists()
    {
        std::fs::create_dir_all(dir).context("failed to create bin dir")?;
    }
    Ok(())
}

fn systemctl(cmd: &[&str]) -> Result<()> {
    let ok = std::process::Command::new("systemctl")
        .args(cmd)
        .status()
        .context("failed to execute systemctl")?
        .success();
    ensure!(ok, "systemctl command {cmd:?} failed");

    Ok(())
}

fn systemctl_service(cmd: &[&str]) -> Result<()> {
    let cmd: Vec<_> = cmd
        .iter()
        .chain([SYSTEMD_SERVICE_NAME].iter())
        .copied()
        .collect();
    systemctl(&cmd)
}
