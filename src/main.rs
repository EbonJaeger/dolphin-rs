mod config;
mod discord;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use clap::{App, Arg};
use serenity::prelude::*;
use simplelog::*;
use std::error::Error;
use std::process;

type ResultBase = Result<(), Box<dyn Error>>;

#[tokio::main()]
async fn main() -> ResultBase {
    let matches = App::new("dolphin")
        .about("Discord and Minecraft chat bridge (re-)written in Rust")
        .author("Evan Maddock <maddock.evan@vivaldi.net>")
        .version(crate_version!())
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .help("Print extra debug messages to stdout"),
        )
        .get_matches();

    let log_level = if matches.is_present("debug") {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    TermLogger::init(log_level, Config::default(), TerminalMode::Mixed).unwrap();

    let cfg: config::RootConfig = confy::load("dolphin")?;
    info!("Config loaded successfully");

    if cfg.discord_config.bot_token == "" {
        warn!("+-----------------------------------------------------------------------------------------------+");
        warn!("| No Discord bot token is configured!                                                           |");
        warn!("|                                                                                               |");
        warn!("| Create a Discord bot here:                                                                    |");
        warn!("| https://discordapp.com/developers/applications/me                                             |");
        warn!("|                                                                                               |");
        warn!("| Copy the token into your config file, and add the bot to your server with this URL:           |");
        warn!("| https://discordapp.com/oauth2/authorize?client_id=<BOT CLIENT ID>&permissions=10240&scope=bot |");
        warn!("+-----------------------------------------------------------------------------------------------+");
        process::exit(0);
    }

    let handler = discord::Handler::new(cfg.clone());

    let mut client = match Client::new(&cfg.discord_config.bot_token)
        .event_handler(handler)
        .await
    {
        Ok(client) => client,
        Err(e) => {
            error!("Error starting Discord client: {}", e);
            process::exit(0);
        }
    };

    if let Err(e) = client.start().await {
        error!("Error starting Discord client: {}", e);
        process::exit(0);
    }

    Ok(())
}
