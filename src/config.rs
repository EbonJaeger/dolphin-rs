extern crate confy;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RootConfig {
    pub discord_config: DiscordConfig,
    pub minecraft_config: MinecraftConfig,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub bot_token: String,
    pub channel_id: u64,
    pub allow_mentions: bool,
    pub use_member_nicks: bool,
    pub webhook_config: WebhookConfig,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub url: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MinecraftConfig {
    pub rcon_ip: String,
    pub rcon_port: i32,
    pub rcon_password: String,
    pub custom_death_keywords: Vec<String>,
    pub log_file_path: String,
    pub templates: TellrawTemplates,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TellrawTemplates {
    pub username_template: String,
    pub attachment_template: String,
    pub message_template: String,
}

impl Default for RootConfig {
    fn default() -> Self {
        RootConfig {
            discord_config: DiscordConfig {
                bot_token: String::new(),
                channel_id: 0,
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
                custom_death_keywords: Vec::new(),
                log_file_path: String::new(),
                templates: TellrawTemplates {
                    username_template: String::from("{\"color\": \"white\", \"text\": \"<%username%> \", \"clickEvent\":{\"action\":\"suggest_command\", \"value\":\"%mention% \"}}",),
                    attachment_template: String::from("{\"color\":\"gray\",\"text\":\"[%num% attachment(s) sent]\", \"clickEvent\":{\"action\":\"open_url\",\"value\":\"%url%\"},\"hoverEvent\":{\"action\":\"show_text\",\"value\":{\"text\":\"Click to open\"}}}"),
                    message_template: String::from("{\"color\":\"white\", \"text\":\"%text%\"}"),
                },
            },
        }
    }
}
