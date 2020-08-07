mod config;

#[macro_use]
extern crate log;

use simplelog::*;

fn main() -> Result<(), confy::ConfyError> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap();

    let _cfg: config::RootConfig = confy::load("dolphin")?;
    info!("Config loaded successfully");

    Ok(())
}
