use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub cmd: Option<Command>,
}

#[derive(Debug, Clone, Copy, Parser)]
pub enum Command {
    /// Install and enable systemd service, install current_exe
    Install,
    /// Install current_exe and restart systemd service
    Update,
}
