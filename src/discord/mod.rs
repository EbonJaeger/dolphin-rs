use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

use crate::config::RootConfig;
use crate::listener::{split_webhook_url, Listener, LogTailer, Webserver};

use rcon::Connection;
use serenity::all::{ChannelId, Interaction};
use serenity::builder::{CreateCommand, CreateInteractionResponseMessage};
use serenity::gateway::ActivityData;
use serenity::utils::parse_channel_mention;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::GuildId},
    prelude::*,
};
use thiserror::Error;
use tracing::{debug, error, info};

mod commands;
mod markdown;

const MAX_LINE_LENGTH: usize = 100;

pub struct Handler {
    config_lock: Arc<RwLock<RootConfig>>,
    guild_id: AtomicU64,
    is_watching: AtomicBool,
}

impl Handler {
    pub fn new(config_lock: Arc<RwLock<RootConfig>>) -> Self {
        Self {
            config_lock,
            guild_id: AtomicU64::new(0),
            is_watching: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            match command.data.name.as_str() {
                "list" => {
                    if let Err(e) = commands::minecraft::list(ctx, command).await {
                        error!("Error performing 'list' command: {}", e);
                    }
                }
                _ => {
                    let response =
                        CreateInteractionResponseMessage::new().content("Unknown command");
                    if let Err(e) = command
                        .create_response(
                            &ctx.http,
                            serenity::builder::CreateInteractionResponse::Message(response),
                        )
                        .await
                    {
                        error!("Error sending interaction response: {}", e);
                    }
                }
            };
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let configured_id = self.config_lock.read().await.get_channel_id();

        // Ignore messages that aren't from the configured channel
        if msg.channel_id.get() != configured_id {
            return;
        }

        // Get our bot user
        let bot = ctx.cache.current_user().clone();

        // Ignore messages that are from ourselves
        let webhook_url = self.config_lock.read().await.webhook_url();
        let webhook_id = split_webhook_url(&webhook_url).unwrap_or_default().0;
        if msg.author.id == bot.id
            || (msg.webhook_id.is_some() && msg.webhook_id.unwrap() == webhook_id)
        {
            debug!("event_handler:message: skipping message from ourselves or our webhook");
            return;
        }

        debug!("event_handler:message: received a message from Discord");
        let content = sanitize_message(&ctx, &msg).await;

        // Send a separate message for each line
        let lines = content.split('\n');

        // Parse and convert any Markdown
        let mut marked = Vec::new();
        lines.for_each(|line| {
            let blocks = markdown::parse(line);
            debug!("event_handler:message: parsed plocks: {:?}", blocks);
            marked.push(markdown::to_minecraft_format(&blocks));
        });

        let lines = truncate_lines(marked);
        let mut lines =
            apply_line_template(self.config_lock.read().await.get_message_template(), lines);

        // Add attachement message if an attachment is present
        if !msg.attachments.is_empty() {
            let line = self.config_lock.read().await.get_attachment_template();
            let line = line.replace("%num%", &msg.attachments.len().to_string());
            let line = line.replace("%url%", &msg.attachments.first().unwrap().url);
            lines.push(line);
        }

        // Get the name to use for these messages
        let name = if self.config_lock.read().await.use_member_nicks() {
            msg.author
                .nick_in(&ctx, msg.guild_id.unwrap())
                .await
                .unwrap_or_else(|| msg.author.name.clone())
        } else {
            msg.author.name.clone()
        };

        // Send each line to Minecraft
        for line in lines {
            let command = build_tellraw_command(
                name.clone(),
                &msg.author.tag(),
                &self.config_lock.read().await.get_username_template(),
                &line,
            );

            if let Err(e) = send_to_minecraft(
                command,
                self.config_lock.read().await.get_rcon_addr(),
                self.config_lock.read().await.get_rcon_password(),
            )
            .await
            {
                error!("Error sending a chat message to Minecraft: {}", e);
            }
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        info!("Connected to Discord");
        let activity_data = ActivityData::playing("Type !help for command list");
        ctx.set_activity(Some(activity_data));
    }

    ///
    /// Use this function to set up and start our Minecraft log watcher.
    ///
    /// We use `cache_ready` instead of just `ready` because this will
    /// involve using things in the cache, so best wait for it to be ready
    /// with this function.
    ///
    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        let ctx = Arc::new(ctx);
        let config_lock = Arc::clone(&self.config_lock);

        if self.guild_id.load(Ordering::Relaxed) == 0 {
            self.guild_id.store(guilds[0].get(), Ordering::Relaxed);
        }

        let guild_id = self.guild_id.load(Ordering::Relaxed);
        let guild_id = Arc::new(GuildId::new(guild_id));
        let log_path = config_lock.read().await.get_log_path();

        // Setup command interactions
        let commands = vec![
            CreateCommand::new("list").description("List all players on the Minecraft server")
        ];
        match guild_id.set_commands(&ctx.http, commands).await {
            Ok(_) => info!("Command interactions registered"),
            Err(e) => error!("Error registering commands: {}", e),
        };

        // Only do stuff if we're not already running
        let loaded = self.is_watching.load(Ordering::Relaxed);
        if !loaded {
            // Create our listener and start waiting for messages
            let enable_webserver = config_lock.read().await.enable_webserver();
            if enable_webserver {
                let port = config_lock.read().await.get_webserver_port();
                tokio::spawn(async move {
                    let listener = Webserver::new(port);
                    listener
                        .listen(ctx.clone(), config_lock.clone(), guild_id.clone())
                        .await;
                });
            } else {
                let log_tailer = LogTailer::new(log_path.to_string());
                tokio::spawn(async move {
                    log_tailer
                        .listen(ctx.clone(), config_lock.clone(), guild_id.clone())
                        .await
                });
            }
        }

        self.is_watching.swap(true, Ordering::Relaxed);
    }
}

///
/// Put each given line into a JSON structure to be passed to the
/// Minecraft tellraw command.
///
fn apply_line_template(template: String, lines: Vec<String>) -> Vec<String> {
    let mut formatted_lines: Vec<String> = Vec::new();

    for line in lines {
        let formatted = template.replace("%content%", line.as_str());
        formatted_lines.push(formatted);
    }

    formatted_lines
}

///
/// Create the tellraw command string from the configured template.
/// This will insert values into the various supported placeholders,
/// returning the final result.
///
fn build_tellraw_command(
    name: String,
    mention: &str,
    username_template: &str,
    content: &str,
) -> String {
    let command = format!("tellraw @a [{}, {}]", username_template, content);

    // Fill in our placeholders
    let command = command.replace("%username%", &name);
    command.replace("%mention%", format!("@{}", mention).as_str())
}

///
/// Performs some string replacements for mentions and escapes quotes on
/// messages that are to be sent to the Minecraft server.
///
async fn sanitize_message(ctx: &Context, msg: &Message) -> String {
    let content = msg.content.clone();
    let mut sanitized = msg.content.clone();

    // We have to do all this nonsense for channel mentions because
    // the Discord API devs are braindead.
    let channel_ids: Vec<ChannelId> = content
        .split_whitespace()
        .filter_map(parse_channel_mention)
        .collect();

    for id in channel_ids {
        if let Some(channel) = ctx.cache.channel(id) {
            sanitized = sanitized.replace(
                format!("<#{}>", id).as_str(),
                format!("#{}", channel.name()).as_str(),
            );
        }
    }

    for role_id in &msg.mention_roles {
        if let Some(role) = role_id.to_role_cached(&ctx.cache) {
            sanitized = sanitized.replace(
                &role_id.mention().to_string(),
                format!("@{}", role.name).as_str(),
            );
        }
    }

    for user_mention in &msg.mentions {
        sanitized = sanitized.replace(
            format!("<@!{}>", user_mention.id).as_str(),
            format!("@{}", user_mention.name).as_str(),
        );
    }

    // Escape double quotes
    sanitized.replace("\"", "\\\"")
}

/// Send a tellraw message to the Minecraft server via RCON. Content
/// should be a valid JSON Object that the game can parse and display.
///
/// If there is an error connecting to RCON or sending the message, the
/// error will be returned.
///
/// # Examples
///
/// ```rust
/// let command = "say Hello, world!";
/// let rcon_addr = "localhost:25575";
/// let rcon_password = "hunter2";
///
/// send_to_minecraft(command, rcon_addr, rcon_password).await?
/// ```
async fn send_to_minecraft(
    command: String,
    rcon_addr: String,
    rcon_password: String,
) -> Result<String, Error> {
    debug!("send_to_minecraft: {}", command);

    // Create RCON connection
    let mut conn = Connection::builder()
        .enable_minecraft_quirks(true)
        .connect(rcon_addr, &rcon_password)
        .await?;

    // Send the command to Minecraft
    let resp = conn.cmd(&command).await?;
    Ok(resp)
}

///
/// Truncates each line if it is longer than the maximum number of characters,
/// by default 100. If a line is over the limit, it will be split at that
/// number of chacacters, and a new line inserted into the line Vector.
///
fn truncate_lines(lines: Vec<String>) -> Vec<String> {
    let mut truncated: Vec<String> = Vec::new();

    for mut line in lines {
        while !line.is_empty() {
            // Push 100 characters to our Vector if the line is longer
            // than 100 characters. If the line is less than that, push
            // the entire line.
            let trunk = match line.get(..MAX_LINE_LENGTH) {
                Some(trunk) => trunk,
                None => &line,
            };

            truncated.push(trunk.to_string());

            // Shorten the line for the next iteration
            line = match line.get(MAX_LINE_LENGTH..) {
                Some(sub) => sub.to_string(),
                None => String::new(),
            };
        }
    }

    truncated
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("rcon error: {0}")]
    Rcon(#[from] rcon::Error),
}

#[cfg(test)]
mod tests {
    use crate::discord::truncate_lines;

    #[test]
    fn split_long_line() {
        // Given
        let input = vec!(String::from("01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789"));
        let expected = vec!("0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789", "0123456789");

        // When
        let result = truncate_lines(input);

        // Then
        assert_eq!(result, expected);
    }

    #[test]
    fn no_split_line() {
        // Given
        let input = vec!(String::from("0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789"));
        let expected = vec!("0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789");

        // When
        let result = truncate_lines(input);

        // Then
        assert_eq!(result, expected);
    }
}
