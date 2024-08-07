use std::sync::Arc;

use crate::config::RootConfig;
use fancy_regex::Regex;
use linemux::MuxedLines;
use serenity::{
    all::WebhookId,
    async_trait,
    builder::ExecuteWebhook,
    client::Context,
    futures::StreamExt,
    model::id::{ChannelId, GuildId},
    prelude::RwLock,
};
use thiserror::Error;
use tracing::{debug, error, info};
use warp::Filter;

use self::parser::{MinecraftMessage, Source};

mod parser;

/// A Listener listens or watches for new messages from a Minecraft instance,
/// depending on the implementation.
#[async_trait]
pub trait Listener {
    /// Begin listening for messages from Minecraft. Usually you'll want to
    /// call this from an async thread so it doesn't block the rest of the
    /// program.
    async fn listen(
        &self,
        ctx: Arc<Context>,
        config_lock: Arc<RwLock<RootConfig>>,
        guild_id: Arc<GuildId>,
    );
}

/// Registers a file event listener to watch for new lines to be added
/// to a file at a given path.
///
/// # Examples
///
/// ```rust
/// let log_tailer = LogTailer::new("/home/minecraft/server/logs/latest.log");
/// tokio::spawn(async move { log_tailer.listen(ctx.clone(), cfg.clone(), guild_id.clone()).await });
/// ```
pub struct LogTailer {
    path: String,
}

impl LogTailer {
    pub fn new(path: String) -> Self {
        LogTailer { path }
    }
}

#[async_trait]
impl Listener for LogTailer {
    async fn listen(
        &self,
        ctx: Arc<Context>,
        config_lock: Arc<RwLock<RootConfig>>,
        guild_id: Arc<GuildId>,
    ) {
        info!("log_tailer:listen: using log file at '{}'", self.path);
        let config = config_lock.read().await;
        let mut parser = parser::MessageParser::new(
            config.get_death_keywords(),
            config.get_death_ignore_keywords(),
        );

        // Create our log watcher
        let mut log_watcher = MuxedLines::new().expect("Unable to create line muxer");
        log_watcher
            .add_file(&self.path)
            .await
            .expect("Unable to add the Minecraft log file to tail");

        info!("log_tailer:listen: started watching the Minecraft log file");

        let regex = config.get_chat_regex();

        // Wait for the next line
        while let Some(Ok(line)) = log_watcher.next().await {
            // Check if the line is something we have to send
            let message = match parser.parse_line(line.line(), regex.clone()).await {
                Some(message) => message,
                None => continue,
            };

            // Send the message to the Discord channel
            if let Err(e) =
                send_to_discord(ctx.clone(), config_lock.clone(), guild_id.clone(), message).await
            {
                error!(
                    "discord:handler: unable to send a message to Discord: {}",
                    e
                );
            };
        }
    }
}

/// Binds to an IP address and port to listen for messages over a network.
/// It watches for messages at the `/message` endpoint.
///
/// # Examples
///
/// ```rust
/// let listener = Webserver::new(25585);
/// listener.listen(ctx.clone(), cfg.clone(), guild_id.clone()).await;
/// ```
pub struct Webserver {
    port: u16,
}

impl Webserver {
    pub fn new(port: u16) -> Self {
        Webserver { port }
    }
}

#[async_trait]
impl Listener for Webserver {
    async fn listen(
        &self,
        ctx: Arc<Context>,
        config_lock: Arc<RwLock<RootConfig>>,
        guild_id: Arc<GuildId>,
    ) {
        // POST /message/:msg
        let messages = warp::post()
            .and(warp::path("message"))
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            .and_then(move |message: MinecraftMessage| {
                let ctx = ctx.clone();
                let cfg = config_lock.clone();
                let guild_id = guild_id.clone();

                // Send the message to the Discord channel
                async move {
                    match send_to_discord(ctx, cfg, guild_id, message).await {
                        Ok(()) => Ok(""),
                        Err(e) => {
                            error!(
                                "discord:handler: unable to send a message to Discord: {}",
                                e
                            );
                            Err(warp::reject::reject())
                        }
                    }
                }
            });

        // TODO: Maybe figure out how to bind to a configurable address?
        warp::serve(messages).run(([0, 0, 0, 0], self.port)).await
    }
}

/// Post a message to the configured Discord webhook.
/// If the message is from a player, we will execute the
/// webhook with that player's head as the avatar and their
/// in-game name as the username.
async fn post_to_webhook(
    ctx: Arc<Context>,
    message: MinecraftMessage,
    url: &str,
) -> Result<(), Error> {
    // Split the url into the webhook id an token
    let parts = match split_webhook_url(url) {
        Some(parts) => parts,
        None => return Err(Error::Webhook(String::from("invalid webhook url"))),
    };

    // Get the webhook using the id and token
    let webhook = ctx
        .http
        .get_webhook_with_token(WebhookId::new(parts.0), parts.1)
        .await?;

    // Get the avatar URL
    let avatar_url = match message.source {
        Source::Player => format!(
            "https://crafatar.com/avatars/{}?size=256",
            message.uuid.clone()
        ),
        // TODO: Do something better than a blind unwrap() here
        Source::Server => ctx.cache.current_user().avatar_url().unwrap(),
    };

    // Build the post content
    let content = ExecuteWebhook::new()
        .avatar_url(avatar_url)
        .username(message.name)
        .content(message.content);

    // Post to the webhook
    webhook.execute(&ctx.http, false, content).await?;

    Ok(())
}

/// Send a message from a Minecraft server to a configured Discord channel, either
/// directly as a message or via a webhook integration.
///
/// # Errors
///
/// Returns a `serenity::Error` if a message is unable to be sent to the channel or the webhook.
async fn send_to_discord(
    ctx: Arc<Context>,
    config_lock: Arc<RwLock<RootConfig>>,
    guild_id: Arc<GuildId>,
    mut message: MinecraftMessage,
) -> Result<(), Error> {
    debug!(
        "dolphin:send_to_discord: received a message from a Minecraft instance: {:?}",
        message
    );

    let config = config_lock.read().await;

    // Set the source name to that of the bot if it's a server message
    if message.source == Source::Server {
        message.name.clone_from(&ctx.cache.current_user().name);
    }

    // Optionally replace mentions in the message
    if config.mentions_allowed() {
        if let Err(e) = message.replace_mentions(ctx.clone(), guild_id) {
            return Err(Error::Parser(e));
        };
    }

    // Check if we should use a webhook to post the message
    let webhook_url = config.webhook_url();
    if !webhook_url.is_empty() {
        post_to_webhook(ctx.clone(), message, &webhook_url).await?
    } else {
        // Send the message to the channel
        let final_msg = match message.source {
            Source::Player => format!("**{}**: {}", message.name, message.content),
            Source::Server => message.content,
        };

        let id = config.get_channel_id();
        ChannelId::new(id).say(&ctx, final_msg).await?;
    }

    Ok(())
}

/// Use Regex to split the configured webhook URL into an ID and a token.
/// If the input url doesn't match the regex, [None] will be returned. No
/// validation is done to see if the webhook URL is actually a valid and
/// active Discord webhook.
///
/// # Examples
///
/// ```rust
/// let webhook_url = String::from("https://discord.com/api/webhooks/12345/67890");
/// let webhook_parts = split_webhook_url(&webhook_url);
///
/// assert!(webhook_parts.is_some());
/// assert_eq!(webhook_parts.unwrap().0, 12345);
/// assert_eq!(webhook_parts.unwrap().1, 67890);
/// ```
pub fn split_webhook_url(url: &str) -> Option<(u64, &str)> {
    // Only compile the regex once, since this is expensive
    lazy_static! {
        static ref WEBHOOK_REGEX: Regex =
            Regex::new(r"^https://discord.com/api/webhooks/(?P<id>.*)/(?P<token>.*)$").unwrap();
    }

    let mut ret = None;

    if let Ok(Some(captures)) = WEBHOOK_REGEX.captures(url) {
        if captures.len() != 3 {
            return None;
        }

        let id = captures.name("id").unwrap().as_str();
        if let Ok(id) = id.parse::<u64>() {
            ret = Some((id, captures.name("token").unwrap().as_str()));
        }
    }

    ret
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Discord error: {0}")]
    Discord(#[from] serenity::Error),

    #[error("parser error: {0}")]
    Parser(#[from] parser::Error),

    #[error("webhook error: {0}")]
    Webhook(String),
}

#[cfg(test)]
mod tests {
    use crate::listener::split_webhook_url;

    #[test]
    fn parse_parts_from_webhook_url() {
        // Given
        let input = String::from("https://discord.com/api/webhooks/12345/67890");

        // When/Then
        match split_webhook_url(&input) {
            Some((id, token)) => {
                assert_eq!(id, 12345);
                assert_eq!(token, String::from("67890"));
            }
            None => panic!("failed to parse Discord webhook url"),
        }
    }

    #[test]
    fn parse_non_webhook_url() {
        // Given
        let input = String::from("https://example.com");

        // When/Then
        if let Some(_) = split_webhook_url(&input) {
            panic!("webhook split returned something when it should have returned None");
        }
    }
}
