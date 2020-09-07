use super::config;
use super::minecraft::{MinecraftMessage, MinecraftWatcher};

use err_derive::Error;
use rcon::Connection;
use serenity::{
    model::{channel::Message, gateway::Activity, gateway::Ready},
    prelude::*,
};
use std::str::Split;
use std::sync::mpsc;
use std::thread;

const MAX_LINE_LENGTH: usize = 100;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "{}", _0)]
    Discord(#[error(source)] serenity::Error),
    #[error(display = "{}", _0)]
    Io(#[error(source)] std::io::Error),
    #[error(display = "{}", _0)]
    Rcon(#[error(source)] rcon::Error),
}

pub struct DiscordBot {
    cfg: config::RootConfig,
    client: Client,
}

impl DiscordBot {
    /// Create a new Discord bot. This will set up the Discord client and event
    /// handler, but will not connect to Discord.
    pub async fn new(cfg: config::RootConfig) -> Result<Self, Error> {
        let handler = Handler::new(cfg.clone());

        // Create the Discord client
        let client = match Client::new(&cfg.discord_config.bot_token)
            .event_handler(handler)
            .await
        {
            Ok(client) => client,
            Err(e) => return Err(Error::Discord(e)),
        };

        Ok(Self { cfg, client })
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        // Create the Minecraft log tailer
        let mut minecraft_watcher = match MinecraftWatcher::new(
            self.cfg.minecraft_config.custom_death_keywords.clone(),
            self.cfg.minecraft_config.log_file_path.clone(),
        )
        .await
        {
            Ok(watcher) => watcher,
            Err(e) => return Err(Error::Io(e)),
        };

        let (tx, rx) = mpsc::channel();

        // Start tailing the Minecraft log file. This is done in a separate thread
        // so the Discord client doesn't get blocked.
        thread::spawn(move || {
            let tx = tx.clone();
            // Spawn a new thread from this thread to start tailing the log file
            // FIXME: This doesn't actually seem to work with the `async move`
            debug!("Starting log watcher thread");
            thread::spawn(move || async move {
                info!("Starting Minecraft log watcher");
                while let Some(message) = minecraft_watcher.read_line().await {
                    if let Err(e) = tx.send(message) {
                        warn!("Error sending a message through the channel: {}", e);
                    }
                }
            });

            // Continuously read from the receiver to get new messages
            while let Ok(message) = rx.recv() {
                debug!(
                    "Received a message from the Minecraft watcher: {}",
                    message.message
                );
            }
        });

        // Connect to Discord and wait for events
        if let Err(e) = self.client.start().await {
            Err(Error::Discord(e))
        } else {
            Ok(())
        }
    }
}

struct Handler {
    cfg: config::RootConfig,
}

impl Handler {
    fn new(cfg: config::RootConfig) -> Self {
        Self { cfg }
    }

    async fn send_to_minecraft(&self, name: &str, content: &str) -> Result<(), Error> {
        let command = format!("tellraw @a {}", self.cfg.minecraft_config.tellraw_template);
        let command = str::replace(command.as_str(), "%username%", name);
        let command = str::replace(command.as_str(), "%message%", content);

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
            Err(e) => return Err(Error::Rcon(e)),
        };

        // Send the command to Minecraft
        match conn.cmd(command.as_str()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Rcon(e)),
        }
    }

    fn truncate_lines<'a>(&self, lines: Split<'a, &'a str>) -> Vec<&'a str> {
        let mut truncated: Vec<&'a str> = Vec::new();

        for mut line in lines {
            while line.len() > 0 {
                // Push 100 characters to our Vector if the line is longer
                // than 100 characters. If the line is less than that, push
                // the entire line.
                let trunk = match line.get(..MAX_LINE_LENGTH) {
                    Some(trunk) => trunk,
                    None => &line,
                };

                truncated.push(trunk);

                // Shorten the line for the next iteration
                line = match line.get(MAX_LINE_LENGTH..) {
                    Some(sub) => sub,
                    None => "",
                };
            }
        }

        truncated
    }
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let configured_id = match self.cfg.discord_config.channel_id.parse::<u64>() {
            Ok(id) => id,
            Err(e) => {
                error!("Error parsing Discord channel ID: {}", e);
                return;
            }
        };

        // Ignore messages that aren't from the configured channel
        if msg.channel_id.as_u64() != &configured_id {
            return;
        }

        // Get our bot user
        let bot = match ctx.http.get_current_user().await {
            Ok(user) => user,
            Err(e) => {
                error!("Error getting current user from Discord: {}", e);
                return;
            }
        };

        // Ignore messages that are from ourselves
        if msg.author.id == bot.id || msg.webhook_id.is_some() {
            debug!("Skipping message from ourselves or webhook");
            return;
        }

        debug!("Received a message from Discord");

        // Get the sender's name to send to Minecraft
        let name = if self.cfg.discord_config.use_member_nicks {
            match msg.author_nick(ctx).await {
                Some(nick) => nick,
                None => msg.author.name,
            }
        } else {
            msg.author.name
        };

        let content = msg.content;

        // Check if the message just consists of an attachment
        if msg.attachments.len() > 0 {
            if content.len() == 0 {
                // Get the URL to the first attachment
                let content = match msg.attachments.get(0) {
                    Some(attachment) => attachment.clone().url,
                    None => String::new(),
                };
                if content.len() > 0 {
                    debug!("Sending an attachment URL to Minecraft");
                    match self.send_to_minecraft(&name, &content).await {
                        Ok(_) => return,
                        Err(e) => {
                            error!("Error sending a chat message to Minecraft: {}", e);
                            return;
                        }
                    };
                }
            }
        }

        // Send a separate message for each line
        let lines = content.split("\n");
        let lines = self.truncate_lines(lines);
        for (index, line) in lines.iter().enumerate() {
            debug!(
                "Sending a chat message to Minecraft: Part {}/{}",
                index + 1,
                lines.len()
            );
            match self.send_to_minecraft(&name, &line).await {
                Ok(_) => continue,
                Err(e) => {
                    error!("Error sending a chat message to Minecraft: {}", e);
                    continue;
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        info!("Connected to Discord");
        ctx.set_activity(Activity::playing("Type !help for command list"))
            .await;
    }
}
