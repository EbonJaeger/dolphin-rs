use std::path::PathBuf;

use clap::{Parser, Subcommand};
use thiserror::Error;

mod start;

#[derive(Parser)]
#[command(author = "Evan Maddock")]
#[command(version = None)]
#[command(about = "Connects your Minecraft server chat to Discord")]
#[command(
    long_about = "Dolphin acts as a bridge between a Discord channel and a Minecraft server; messages sent in one place will be passed on to the other."
)]
#[command(arg_required_else_help = true)]
struct Cli {
    /// Creates or loads the configuration at the given path
    #[arg(short = 'c', long = "config", value_name = "FILE")]
    config: Option<PathBuf>,

    /// Print extra debug messages to stdout
    #[arg(short = 'd', long = "debug")]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to Discord and start reading the Minecraft log file
    Start {},
}

pub async fn process() -> Result<(), Error> {
    let cli = Cli::parse();

    // Get the configuration file path to use
    let config_path = match cli.config {
        Some(config) => config,
        None => confy::get_configuration_file_path("dolphin", "dolphin")?,
    };

    // Handle the proper subcommand
    match cli.command {
        Some(Commands::Start {}) => start::handle(config_path.clone(), cli.debug)
            .await
            .map_err(Error::Start),
        _ => unreachable!(),
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("config")]
    Config(#[from] confy::ConfyError),

    #[error("start")]
    Start(#[from] start::Error),
}
