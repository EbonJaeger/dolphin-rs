use crate::config::RootConfig;
use crate::errors::DolphinError;
use crate::minecraft::{MessageParser, MinecraftMessage, Source};
use linemux::MuxedLines;
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
};
use std::{
    str::Split,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
use tokio::stream::StreamExt;
use tracing::{debug, error, info};

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
            let formatted = self.cfg.minecraft_config.templates.message_template.clone();
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
            self.cfg.minecraft_config.templates.username_template, content
        );

        // Get the sender's name to send to Minecraft
        let name = if self.cfg.discord_config.use_member_nicks {
            author
                .nick_in(&ctx.http, msg.guild_id.unwrap())
                .await
                .unwrap_or_else(|| author.name.clone())
        } else {
            author.name.clone()
        };

        // Fill in our placeholders
        let command = command.replace("%username%", &name);
        command.replace("%mention%", &author.mention())
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
        debug!("{}", command);

        // Create RCON connection
        let addr = format!(
            "{}:{}",
            self.cfg.minecraft_config.rcon_ip, self.cfg.minecraft_config.rcon_port
        );

        let mut conn = match Connection::builder()
            .enable_minecraft_quirks(true)
            .connect(addr, self.cfg.minecraft_config.rcon_password.as_str())
            .await
        {
            Ok(conn) => conn,
            Err(e) => return Err(DolphinError::Rcon(e)),
        };

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
        let configured_id = self.cfg.discord_config.channel_id;

        // Ignore messages that aren't from the configured channel
        if msg.channel_id.as_u64() != &configured_id {
            return;
        }

        // Get our bot user
        let bot = ctx.cache.current_user().await;

        // Ignore messages that are from ourselves
        if msg.author.id == bot.id || msg.webhook_id.is_some() {
            debug!("Skipping message from ourselves or webhook");
            return;
        }

        debug!("Received a message from Discord");

        let content = msg.content.clone();

        // Send a separate message for each line
        let lines = content.split("\n");
        let lines = truncate_lines(lines);
        let mut lines = self.apply_line_template(lines);

        // Add attachement message if an attachment is present
        if !msg.attachments.is_empty() {
            let line = self
                .cfg
                .minecraft_config
                .templates
                .attachment_template
                .clone();
            let line = line.replace("%num%", &msg.attachments.len().to_string());
            let line = line.replace("%url%", &msg.attachments.first().unwrap().url);
            lines.push(line);
        }

        // Send each line to Minecraft
        for (index, line) in lines.iter().enumerate() {
            debug!(
                "Sending a chat message to Minecraft: Part {}/{}",
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
        let guild_id = GuildId(guild_id);

        let log_path = &cfg.minecraft_config.log_file_path;
        info!("Using log file at '{}'", log_path);

        // Only do stuff if we're not already running
        if !self.is_watching.load(Ordering::Relaxed) {
            let ctx_cloned = Arc::clone(&ctx);
            let parser = MessageParser::new(cfg.minecraft_config.custom_death_keywords.clone());

            // Create our log watcher
            let mut log_watcher = MuxedLines::new().unwrap();
            log_watcher
                .add_file(&log_path)
                .await
                .expect("Unable to add the Minecraft log file to tail");

            // Spawn a task to continuously tail the log file
            tokio::spawn(async move {
                info!("Started watching the Minecraft log file");
                watch_log_file(
                    ctx_cloned,
                    Arc::clone(&cfg),
                    guild_id,
                    &mut log_watcher,
                    parser,
                )
                .await;
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
/// Watches the Minecraft log file and waits for new lines to be
/// received. When a line is received, it will be parsed and sent
/// to Discord if it's a type of message that we want to broadcast.
///
async fn watch_log_file(
    ctx: Arc<Context>,
    cfg: Arc<RootConfig>,
    guild_id: GuildId,
    log_watcher: &mut MuxedLines,
    parser: MessageParser,
) {
    // Wait for the next line
    while let Some(Ok(line)) = log_watcher.next().await {
        // Turn it into our message struct if it matches something
        // we care about.
        let message = match parser.parse_line(line.line()) {
            Some(message) => message,
            None => continue,
        };

        // Get the correct name to use
        let name = match message.source {
            Source::Player => message.name.clone(),
            Source::Server => ctx.cache.current_user().await.name,
        };

        let mut content = message.content.clone();
        if cfg.discord_config.allow_mentions {
            content = replace_mentions(Arc::clone(&ctx), guild_id, content).await;
        }

        let message = MinecraftMessage {
            name: name.clone(),
            content,
            source: message.source,
        };

        // Check if we should use a webhook to post the message
        if cfg.discord_config.webhook_config.enabled {
            let url = &cfg.discord_config.webhook_config.url;

            if let Err(e) = post_to_webhook(Arc::clone(&ctx), message, url).await {
                error!("Error posting to webhook: {}", e);
            }
        } else {
            // Send the message to the channel
            let final_msg = match message.source {
                Source::Player => format!("**{}**: {}", message.name, message.content),
                Source::Server => message.content,
            };

            if let Err(e) = ChannelId(cfg.discord_config.channel_id)
                .say(&ctx, final_msg)
                .await
            {
                error!("Error sending a message to Discord: {:?}", e);
            }
        }
    }
}

///
/// Looks for instances of user mentions in a message and attempts
/// to replace that text with an actual Discord @mention.
///
async fn replace_mentions(ctx: Arc<Context>, guild_id: GuildId, message: String) -> String {
    let mut cloned = message.clone();

    // Get the members from the Guild
    let members = match ctx.cache.guild_field(guild_id, |g| g.members.clone()).await {
        Some(members) => members,
        None => return cloned,
    };

    /*
     * Split the message on whitespace, and filter out any words that don't
     * start with an '@' symbol. For each word that does, look to see if it
     * matches any of the member names, and replace the original word with
     * their @mention.
     */
    message
        .split_whitespace()
        .filter(|w| w.starts_with('@'))
        .for_each(|m| {
            let name = &m[1..];
            for member in members.values() {
                if member
                    .nick
                    .as_ref()
                    .unwrap_or(&member.user.name)
                    .eq_ignore_ascii_case(name)
                    || member.user.name.eq_ignore_ascii_case(name)
                {
                    cloned = cloned.replace(m, &member.mention());
                    break;
                }
            }
        });

    cloned
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
    let webhook = match ctx.http.get_webhook_with_token(parts.0, parts.1).await {
        Ok(webhook) => webhook,
        Err(e) => return Err(DolphinError::Discord(e)),
    };

    // Get the avatar URL
    let avatar_url = match message.source {
        Source::Player => format!("https://minotar.net/helm/{}/256.png", message.name.clone()),
        Source::Server => ctx.cache.current_user().await.avatar_url().unwrap(),
    };

    // Post to the webhook
    match webhook
        .execute(&ctx.http, false, |w| {
            w.avatar_url(avatar_url)
                .username(message.name)
                .content(message.content)
        })
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(DolphinError::Discord(e)),
    }
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

    Some((
        captures
            .name("id")
            .unwrap()
            .as_str()
            .parse::<u64>()
            .unwrap(),
        captures.name("token").unwrap().as_str(),
    ))
}
