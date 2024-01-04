use std::{env, num::ParseIntError, path::PathBuf, sync::Arc};

use serenity::{all::ApplicationId, prelude::GatewayIntents, Client};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{info, Level};

use crate::{
    config::{
        container::{ConfigContainer, ConfigPathContainer},
        RootConfig,
    },
    discord::Handler,
};

pub async fn handle(config_path: PathBuf, debug: bool) -> Result<(), Error> {
    let log_level = match debug {
        true => Level::DEBUG,
        false => Level::INFO,
    };

    // Set up the tracing logger
    let format = tracing_subscriber::fmt::format()
        .pretty()
        .compact()
        .with_target(false);

    tracing_subscriber::fmt()
        .event_format(format)
        .with_max_level(log_level)
        .init();

    // Load the configuration file
    let config: RootConfig = confy::load_path(&config_path)?;
    confy::store_path(&config_path, &config)?;
    let config_lock = Arc::new(RwLock::new(config));

    info!("Config loaded successfully");

    let bot_token = match env::var("DISCORD_TOKEN") {
        Ok(token) => token,
        _ => return Err(Error::NoToken),
    };

    let application_id: ApplicationId = match env::var("DISCORD_APPLICATION_ID") {
        Ok(id) => match id.parse() {
            Ok(id) => ApplicationId::new(id),
            Err(e) => return Err(Error::Parse(e)),
        },
        _ => return Err(Error::NoApplicationID),
    };

    // Create our Discord handler
    let handler = Handler::new(config_lock.clone());

    // Create our Discord client
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(bot_token, intents)
        .application_id(application_id)
        .event_handler(handler)
        .await?;

    // Put our config into our Discord client data
    {
        let mut data = client.data.write().await;
        data.insert::<ConfigContainer>(config_lock.clone());
        data.insert::<ConfigPathContainer>(Arc::new(config_path));
    }

    // Connect to Discord and wait for events
    info!("Starting Discord client");
    match client.start().await {
        Ok(()) => Ok(()),
        Err(e) => Err(Error::Discord(e)),
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("configuration error")]
    Config(#[from] confy::ConfyError),

    #[error("Discord error")]
    Discord(#[from] serenity::Error),

    #[error("no Discord Application ID given")]
    NoApplicationID,

    #[error("no Discord token given")]
    NoToken,

    #[error("parse error")]
    Parse(#[from] ParseIntError),
}
