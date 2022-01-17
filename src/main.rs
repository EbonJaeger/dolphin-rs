mod commands;
mod config;
mod discord;
mod errors;
mod listener;
mod markdown;
mod minecraft;

extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate pipeline;

use clap::{crate_version, App, Arg};
use config::RootConfig;
use discord::Handler;
use serenity::{
    client::{bridge::gateway::GatewayIntents, validate_token},
    prelude::*,
};
use std::{env, error::Error, process, sync::Arc};
use tracing::{info, warn, Level};

struct ConfigContainer;

impl TypeMapKey for ConfigContainer {
    type Value = Arc<RwLock<RootConfig>>;
}

struct ConfigPathContainer;

impl TypeMapKey for ConfigPathContainer {
    type Value = Arc<String>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("dolphin")
        .about("Discord and Minecraft chat bridge (re-)written in Rust")
        .author("Evan Maddock <maddock.evan@vivaldi.net>")
        .version(crate_version!())
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .help("Creates or loads the configuration at the given path")
                .takes_value(true)
                .value_name("FILE"),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .help("Print extra debug messages to stdout"),
        )
        .get_matches();

    let log_level = if matches.is_present("debug") {
        Level::DEBUG
    } else {
        Level::INFO
    };

    // Set up the logging system
    tracing_subscriber::fmt()
        .pretty()
        .compact()
        .with_target(false)
        .with_max_level(log_level)
        .init();

    let config_path = matches.value_of("config");

    let cfg: config::RootConfig = match config_path {
        Some(path) => confy::load_path(path).expect("Unable to load the configuration file"),
        None => confy::load("dolphin").expect("Unable to load the configuration file"),
    };

    // Save the config back to disk to make sure new options are saved
    match config_path {
        Some(path) => confy::store_path(path, cfg.clone())?,
        None => confy::store("dolphin", cfg.clone())?,
    };

    let cfg_lock = Arc::new(RwLock::new(cfg));
    info!("Config loaded successfully");

    // validate the bot token
    let bot_token = env::var("DISCORD_TOKEN").expect("expected a Discord token in the environment");
    if validate_token(bot_token.clone()).is_err() {
        warn!("+--------------------------------------------------------------------------------------------+");
        warn!("| Discord bot token is either missing or invalid!                                            |");
        warn!("|                                                                                            |");
        warn!("| Create a Discord bot here:                                                                 |");
        warn!("| https://discord.com/developers/applications/me                                             |");
        warn!("|                                                                                            |");
        warn!("| Copy the token into your config file, and add the bot to your server with this URL:        |");
        warn!("| https://discord.com/oauth2/authorize?client_id=<BOT CLIENT ID>&permissions=10240&scope=bot |");
        warn!("+--------------------------------------------------------------------------------------------+");
        process::exit(0);
    }

    let application_id: u64 = env::var("DISCORD_APPLICATION_ID")
        .expect("Expect a Discord application ID in the environment")
        .parse()
        .expect("Application ID is not a valid ID");

    let handler = Handler::new(Arc::clone(&cfg_lock));

    // Create the Discord client
    let mut client = Client::builder(&bot_token)
        .application_id(application_id)
        .event_handler(handler)
        .intents(
            GatewayIntents::GUILDS
                | GatewayIntents::GUILD_MEMBERS
                | GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILD_PRESENCES,
        )
        .await
        .expect("Error creating Discord client");

    {
        let mut data = client.data.write().await;
        data.insert::<ConfigContainer>(Arc::clone(&cfg_lock));

        if let Some(path) = config_path {
            data.insert::<ConfigPathContainer>(Arc::new(path.to_string()));
        } else {
            data.insert::<ConfigPathContainer>(Arc::new(String::new()));
        }
    }

    // Connect to Discord and wait for events
    info!("Starting Discord client");
    if let Err(e) = client.start().await {
        eprintln!("Discord client error: {}", e);
    }

    Ok(())
}
