mod config;
mod discord;
mod minecraft;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use clap::{App, Arg};
use discord::Handler;
use serenity::{client::validate_token, prelude::*};
use simplelog::*;
use std::{error::Error, process};

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

    let cfg: config::RootConfig =
        confy::load("dolphin").expect("Unable to load the configuration file");
    info!("Config loaded successfully");

    // validate the bot token
    let bot_token = &cfg.discord_config.bot_token;
    if validate_token(bot_token).is_err() {
        warn!("+-----------------------------------------------------------------------------------------------+");
        warn!("| Discord bot token is either missing or invalid!                                               |");
        warn!("|                                                                                               |");
        warn!("| Create a Discord bot here:                                                                    |");
        warn!("| https://discordapp.com/developers/applications/me                                             |");
        warn!("|                                                                                               |");
        warn!("| Copy the token into your config file, and add the bot to your server with this URL:           |");
        warn!("| https://discordapp.com/oauth2/authorize?client_id=<BOT CLIENT ID>&permissions=10240&scope=bot |");
        warn!("+-----------------------------------------------------------------------------------------------+");
        process::exit(0);
    }

    let handler = Handler::new(cfg.clone());

    // Create the Discord client
    let mut client = Client::new(&cfg.discord_config.bot_token)
        .event_handler(handler)
        .await
        .expect("Error creating Discord client");

    // Connect to Discord and wait for events
    info!("Starting Discord client");
    if let Err(e) = client.start().await {
        eprintln!("Discord client error: {:?}", e);
    }

    Ok(())
}
