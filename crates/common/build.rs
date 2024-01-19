use std::fs;

use anyhow::Result;
use serde::Deserialize;

fn main() -> Result<()> {
    parse_bot_version()?;
    Ok(())
}

fn parse_bot_version() -> Result<()> {
    const VERSION_CONF: &str = "../../Cargo.toml";

    println!("cargo:rerun-if-changed={VERSION_CONF}");

    let conf: Config = toml::de::from_str(&fs::read_to_string(VERSION_CONF)?)?;
    println!(
        "cargo:rustc-env=BOT_VERSION={}",
        conf.package.metadata.bot.version
    );
    Ok(())
}

// Cargo.toml

#[derive(Debug, Deserialize)]
struct Config {
    package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
    metadata: Metadata,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    bot: Bot,
}

#[derive(Debug, Deserialize)]
struct Bot {
    version: usize,
}
