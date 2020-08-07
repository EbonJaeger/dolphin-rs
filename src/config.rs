extern crate confy;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RootConfig {
    pub discord_config: DiscordConfig,
    pub minecraft_config: MinecraftConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub bot_token: String,
    pub channel_id: String,
    pub allow_mentions: bool,
    pub use_member_nicks: bool,
    pub webhook_config: WebhookConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftConfig {
    pub rcon_ip: String,
    pub rcon_port: i32,
    pub rcon_password: String,
    pub tellraw_template: String,
    pub custom_death_keywords: Vec<String>,
    pub log_file_path: String,
}

impl Default for RootConfig {
    fn default() -> Self {
        RootConfig {
            discord_config: DiscordConfig {
                bot_token: String::new(),
                channel_id: String::new(),
                allow_mentions: true,
                use_member_nicks: false,
                webhook_config: WebhookConfig {
                    enabled: false,
                    url: String::new(),
                },
            },
            minecraft_config: MinecraftConfig {
                rcon_ip: String::from("localhost"),
                rcon_port: 25575,
                rcon_password: String::new(),
                tellraw_template: String::from(
                    "[{\"color\": \"white\", \"text\": \"<%username%> %message%\"}]",
                ),
                custom_death_keywords: Vec::new(),
                log_file_path: String::new(),
            },
        }
    }
}
