use crate::config::RootConfig;
use crate::errors::DolphinError;
use crate::listener::{Listener, LogTailer, Webserver};
use crate::minecraft::{MinecraftMessage, Source};
use rcon::Connection;
use regex::Regex;
use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::{Activity, Ready},
        id::{ChannelId, GuildId},
        user::User,
    },
    prelude::*,
    utils::parse_channel,
};
use std::{
    str::Split,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

const MAX_LINE_LENGTH: usize = 100;

pub struct Handler {
    cfg: Arc<RootConfig>,
    guild_id: AtomicU64,
    is_watching: AtomicBool,
}

impl Handler {
    pub fn new(cfg: Arc<RootConfig>) -> Self {
        Self {
            cfg,
            guild_id: AtomicU64::new(0),
            is_watching: AtomicBool::new(false),
        }
    }

    ///
    /// Put each given line into a JSON structure to be passed to the
    /// Minecraft tellraw command.
    ///
    fn apply_line_template(&self, lines: Vec<String>) -> Vec<String> {
        let mut formatted_lines: Vec<String> = Vec::new();

        for line in lines {
            let formatted = self.cfg.get_message_template();
            let formatted = formatted.replace("%content%", line.as_str());
            formatted_lines.push(formatted);
        }

        formatted_lines
    }

    ///
    /// Create the tellraw command string from the configured template.
    /// This will insert values into the various supported placeholders,
    /// returning the final result.
    ///
    async fn build_tellraw_command(
        &self,
        author: &User,
        ctx: &Context,
        content: &str,
        msg: &Message,
    ) -> String {
        let command = format!(
            "tellraw @a [{}, {}]",
            self.cfg.get_username_template(),
            content
        );

        // Get the sender's name to send to Minecraft
        let name = if self.cfg.use_member_nicks() {
            author
                .nick_in(&ctx, msg.guild_id.unwrap())
                .await
                .unwrap_or_else(|| author.name.clone())
        } else {
            author.name.clone()
        };

        // Fill in our placeholders
        let command = command.replace("%username%", &name);
        command.replace("%mention%", format!("@{}", &author.tag()).as_str())
    }

    ///
    /// Performs some string replacements for mentions and escapes quotes on
    /// messages that are to be sent to the Minecraft server.
    ///
    async fn sanitize_message(&self, ctx: &Context, msg: &Message) -> String {
        let content = msg.content.clone();
        let mut sanitized = msg.content.clone();

        // We have to do all this nonsense for channel mentions because
        // the Discord API devs are braindead.
        let channel_ids: Vec<u64> = content
            .split_whitespace()
            .filter_map(parse_channel)
            .collect();

        for id in channel_ids {
            if let Some(channel) = ctx.cache.guild_channel(id).await {
                sanitized = sanitized.replace(
                    format!("<#{}>", id).as_str(),
                    format!("#{}", channel.name()).as_str(),
                );
            }
        }

        for role_id in &msg.mention_roles {
            if let Some(role) = role_id.to_role_cached(&ctx.cache).await {
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

    ///
    /// Send a tellraw message to the Minecraft server via RCON. Content
    /// should be a valid JSON Object that the game can parse and display.
    ///
    /// If there is an error connecting to RCON or sending the message, the
    /// error will be returned.
    ///
    async fn send_to_minecraft(
        &self,
        author: &User,
        ctx: &Context,
        content: &str,
        msg: &Message,
    ) -> Result<(), DolphinError> {
        let command = self.build_tellraw_command(author, ctx, content, msg).await;
        debug!("send_to_minecraft: {}", command);

        // Create RCON connection
        let addr = self.cfg.get_rcon_addr();

        let mut conn = Connection::builder()
            .enable_minecraft_quirks(true)
            .connect(addr, self.cfg.get_rcon_password().as_str())
            .await?;

        // Send the command to Minecraft
        match conn.cmd(command.as_str()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(DolphinError::Rcon(e)),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let configured_id = self.cfg.get_channel_id();

        // Ignore messages that aren't from the configured channel
        if msg.channel_id.as_u64() != &configured_id {
            return;
        }

        // Get our bot user
        let bot = ctx.cache.current_user().await;

        // Ignore messages that are from ourselves
        if msg.author.id == bot.id || msg.webhook_id.is_some() {
            debug!("event_handler:message: skipping message from ourselves or webhook");
            return;
        }

        debug!("event_handler:message: received a message from Discord");
        let content = self.sanitize_message(&ctx, &msg).await;

        // Send a separate message for each line
        let lines = content.split("\n");
        let lines = truncate_lines(lines);
        let mut lines = self.apply_line_template(lines);

        // Add attachement message if an attachment is present
        if !msg.attachments.is_empty() {
            let line = self.cfg.get_attachment_template();
            let line = line.replace("%num%", &msg.attachments.len().to_string());
            let line = line.replace("%url%", &msg.attachments.first().unwrap().url);
            lines.push(line);
        }

        // Send each line to Minecraft
        for (index, line) in lines.iter().enumerate() {
            debug!(
                "event_handler:message: sending a chat message to Minecraft: part {}/{}",
                index + 1,
                lines.len()
            );
            if let Err(e) = self.send_to_minecraft(&msg.author, &ctx, &line, &msg).await {
                error!("Error sending a chat message to Minecraft: {}", e);
            }
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        info!("Connected to Discord");
        ctx.set_activity(Activity::playing("Type !help for command list"))
            .await;
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
        let cfg = Arc::clone(&self.cfg);

        if self.guild_id.load(Ordering::Relaxed) == 0 {
            self.guild_id.store(guilds[0].0, Ordering::Relaxed);
        }

        let guild_id = self.guild_id.load(Ordering::Relaxed);
        let guild_id = Arc::new(GuildId(guild_id));
        let log_path = &cfg.get_log_path();

        // Only do stuff if we're not already running
        if !self.is_watching.load(Ordering::Relaxed) {
            let (tx, mut rx) = mpsc::channel(100);

            // Create our listener and start waiting for messages
            if cfg.enable_webserver() {
                let port = cfg.get_webserver_port();
                tokio::spawn(async move {
                    let listener = Webserver::new(port);
                    listener.listen(tx).await;
                });
            } else {
                let log_tailer = LogTailer::new(log_path.to_string(), cfg.get_death_keywords());
                tokio::spawn(async move { log_tailer.listen(tx).await });
            }

            // Spawn a task to wait for messages and send them to Discord
            tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    if let Err(e) = send_to_discord(
                        Arc::clone(&ctx),
                        Arc::clone(&cfg),
                        Arc::clone(&guild_id),
                        message,
                    )
                    .await
                    {
                        error!(
                            "discord:handler: unable to send a message to Discord: {}",
                            e
                        );
                    }
                }
            });
        }

        self.is_watching.swap(true, Ordering::Relaxed);
    }
}

///
/// Truncates each line if it is longer than the maximum number of characters,
/// by default 100. If a line is over the limit, it will be split at that
/// number of chacacters, and a new line inserted into the line Vector.
///
fn truncate_lines<'a>(lines: Split<'a, &'a str>) -> Vec<String> {
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
                Some(sub) => sub,
                None => "",
            };
        }
    }

    truncated
}

///
/// Send a message from a Minecraft server to a configured Discord channel, either
/// directly or via a webhook integration.
///
/// # Errors
///
/// Returns a `serenity::Error` if a message is unable to be sent to the channel or the webhook.
///
async fn send_to_discord(
    ctx: Arc<Context>,
    cfg: Arc<RootConfig>,
    guild_id: Arc<GuildId>,
    message: MinecraftMessage,
) -> Result<(), DolphinError> {
    debug!(
        "dolphin:send_to_discord: received a message from a Minecraft instance: {:?}",
        message
    );

    // Get the correct name to use
    let name = match message.source {
        Source::Player => message.name.clone(),
        Source::Server => ctx.cache.current_user().await.name,
    };

    let mut content = message.content.clone();
    if cfg.mentions_allowed() {
        content = replace_mentions(Arc::clone(&ctx), guild_id, content).await;
    }

    let message = MinecraftMessage {
        name: name.clone(),
        content,
        source: message.source,
    };

    // Check if we should use a webhook to post the message
    if cfg.webhook_enabled() {
        let url = &cfg.webhook_url();

        if let Err(e) = post_to_webhook(Arc::clone(&ctx), message, url).await {
            return Err(e);
        }
    } else {
        // Send the message to the channel
        let final_msg = match message.source {
            Source::Player => format!("**{}**: {}", message.name, message.content),
            Source::Server => message.content,
        };

        if let Err(e) = ChannelId(cfg.get_channel_id()).say(&ctx, final_msg).await {
            return Err(DolphinError::Discord(e));
        }
    }

    Ok(())
}

///
/// Looks for instances of mentions in a message and attempts
/// to replace that text with an actual Discord `@mention` (or
/// `#channel` in the case of a channel).
///
/// It tries to match names using the full name and, in the
/// case of users, optionally their  descriptor. This works
/// for names that have spaces in them, and really probably
/// anything else.
///
async fn replace_mentions(ctx: Arc<Context>, guild_id: Arc<GuildId>, message: String) -> String {
    let mut ret = message.clone();

    if let Some(guild) = ctx.cache.guild(*guild_id).await {
        let mut found_start = false;
        let mut start = 0;
        let mut end = 0;
        let cloned = ret.clone();

        for (i, c) in cloned.char_indices() {
            if !found_start && (c == '@' || c == '#') {
                found_start = true;
                start = i;
            } else if found_start && c == '#' {
                end = i + 5;
            } else if found_start && c == ' ' {
                end = i;
            } else if found_start && cloned.len() == i + 1 {
                end = i + 1;
            }

            // Check to see if we have a mention
            if found_start && end > 0 {
                if let Some(mention) = cloned.get(start..end) {
                    let name = &mention[1..];
                    if let Some(member) = guild.member_named(name) {
                        ret = ret.replace(mention, &member.mention().to_string());
                        start = 0;
                        end = 0;
                        found_start = false;
                    } else if let Some(role) = guild.role_by_name(name) {
                        ret = ret.replace(mention, &role.mention().to_string());
                        start = 0;
                        end = 0;
                        found_start = false;
                    } else if let Some(id) = guild.channel_id_from_name(ctx.clone(), name).await {
                        if let Some(channel) = ctx.cache.channel(id).await {
                            ret = ret.replace(mention, &channel.mention().to_string());
                            start = 0;
                            end = 0;
                            found_start = false;
                        }
                    }
                }
            }
        }
    } else {
        warn!("Unable to get the Guild from the cache: Guild not found");
    }

    ret
}

///
/// Post a message to the configured Discord webhook.
///
async fn post_to_webhook(
    ctx: Arc<Context>,
    message: MinecraftMessage,
    url: &str,
) -> Result<(), DolphinError> {
    // Split the url into the webhook id an token
    let parts = match split_webhook_url(url) {
        Some(parts) => parts,
        None => return Err(DolphinError::Other("invalid webhook url")),
    };

    // Get the webhook using the id and token
    let webhook = ctx.http.get_webhook_with_token(parts.0, parts.1).await?;

    // Get the avatar URL
    let avatar_url = match message.source {
        Source::Player => format!("https://minotar.net/helm/{}/256.png", message.name.clone()),
        // TODO: Do something better than a blind unwrap() here
        Source::Server => ctx.cache.current_user().await.avatar_url().unwrap(),
    };

    // Post to the webhook
    if let Err(e) = webhook
        .execute(&ctx.http, false, |w| {
            w.avatar_url(avatar_url)
                .username(message.name)
                .content(message.content)
        })
        .await
    {
        return Err(DolphinError::Discord(e));
    }

    Ok(())
}

///
/// Use Regex to split the configured webhook URL into an ID and a token.
///
fn split_webhook_url(url: &str) -> Option<(u64, &str)> {
    // Only compile the regex once, since this is expensive
    lazy_static! {
        static ref WEBHOOK_REGEX: Regex =
            Regex::new(r"^https://discord.com/api/webhooks/(?P<id>.*)/(?P<token>.*)$").unwrap();
    }

    let captures = match WEBHOOK_REGEX.captures(&url) {
        Some(captures) => captures,
        None => return None,
    };

    if captures.len() != 3 {
        return None;
    }

    let id = captures.name("id").unwrap().as_str();
    let id = match id.parse::<u64>() {
        Ok(num) => num,
        Err(_) => return None,
    };

    Some((id, captures.name("token").unwrap().as_str()))
}

#[cfg(test)]
mod tests {
    use crate::discord::split_webhook_url;
    use crate::discord::truncate_lines;

    #[test]
    fn split_long_line() {
        // Given
        let input = String::from("01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789");
        let split = input.split("\n");
        let expected = vec!("0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789", "0123456789");

        // When
        let result = truncate_lines(split);

        // Then
        assert_eq!(result, expected);
    }

    #[test]
    fn no_split_line() {
        // Given
        let input = String::from("0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789");
        let split = input.split("\n");
        let expected = vec!("0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789");

        // When
        let result = truncate_lines(split);

        // Then
        assert_eq!(result, expected);
    }

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
