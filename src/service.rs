use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};

const SYSTEMD_SERVICE: &str = include_str!("../assets/bot.service");
const SYSTEMD_SERVICE_PATH: &str = "/etc/systemd/system/app-pulse-bot.service";
const SYSTEMD_SERVICE_NAME: &str = "app-pulse-bot.service";
const BIN_PATH: &str = "/usr/local/bin/app-pulse-bot";

pub fn install() -> Result<()> {
    let service_path = PathBuf::from(SYSTEMD_SERVICE_PATH);
    if service_path.exists() {
        bail!("service already installed");
    }

    ensure_dir(&service_path).context("creating systemd dir")?;
    std::fs::write(service_path, SYSTEMD_SERVICE).context("failed to write systemd unit")?;

    install_bin()?;

    systemctl(&["enable", "--now"]).context("enabling")?;

    Ok(())
}

pub fn update() -> Result<()> {
    install_bin()?;
    systemctl(&["restart"]).context("restarting")?;
    Ok(())
}

fn install_bin() -> Result<()> {
    let bin_path = PathBuf::from(BIN_PATH);
    ensure_dir(&bin_path).context("creating bin dir")?;

    let current_exe = std::env::current_exe().context("failed to get current exe")?;
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
        .arg(SYSTEMD_SERVICE_NAME)
        .status()
        .context("failed to execute systemctl")?
        .success();
    ensure!(ok, "systemctl command {cmd:?} failed");

    Ok(())
}
