use super::config;
use rconmc::{Client, Error};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Activity, gateway::Ready},
    prelude::*,
};
use std::str::Split;

const MAX_LINE_LENGTH: usize = 100;

pub struct Handler {
    cfg: config::RootConfig,
}

impl Handler {
    pub fn new(cfg: config::RootConfig) -> Handler {
        Handler { cfg }
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
        let mut client =
            match Client::dial(addr, self.cfg.minecraft_config.rcon_password.as_str()).await {
                Ok(client) => client,
                Err(e) => return Err(e),
            };

        // Send the command to Minecraft
        match client.send_command(command.as_str()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
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

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // if msg.content == "!ping" {
        //     if let Err(e) = msg.channel_id.say(&ctx.http, "Pong!") {
        //         error!("Error sending a discord message: {:?}", e);
        //     }
        // }

        let configured_id = match self.cfg.discord_config.channel_id.parse::<u64>() {
            Ok(id) => id,
            Err(e) => {
                error!("Error parsing Discord channel ID: {:?}", e);
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
                error!("Error getting current user from Discord: {:?}", e);
                return;
            }
        };

        // Ignore messages that are from ourselves
        if msg.author.id == bot.id || msg.webhook_id.is_some() {
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
        for line in lines {
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
