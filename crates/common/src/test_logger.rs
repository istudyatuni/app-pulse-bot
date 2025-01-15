pub fn init_logger() {
    let _ = simplelog::TermLogger::init(
        log::LevelFilter::Debug,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    )
    .inspect_err(|e| eprintln!("failed to init logger: {e}"));
}
