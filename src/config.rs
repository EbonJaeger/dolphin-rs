extern crate confy;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RootConfig {
    discord_config: DiscordConfig,
    minecraft_config: MinecraftConfig,
    listener_config: ListenerConfig,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    bot_token: String,
    channel_id: u64,
    allow_mentions: bool,
    use_member_nicks: bool,
    webhook_config: WebhookConfig,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    enabled: bool,
    url: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MinecraftConfig {
    rcon_ip: String,
    rcon_port: i32,
    rcon_password: String,
    custom_death_keywords: Vec<String>,
    log_file_path: String,
    templates: TellrawTemplates,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ListenerConfig {
    enabled: bool,
    port: u16,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TellrawTemplates {
    username_template: String,
    attachment_template: String,
    message_template: String,
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
                    message_template: String::from("{\"color\":\"white\", \"text\":\"%content%\"}"),
                },
            },
            listener_config: ListenerConfig {
                enabled: false,
                port: 25585,
            }
        }
    }
}

impl RootConfig {
    pub fn get_bot_token(&self) -> String {
        self.discord_config.bot_token.clone()
    }

    pub fn get_channel_id(&self) -> u64 {
        self.discord_config.channel_id
    }

    pub fn mentions_allowed(&self) -> bool {
        self.discord_config.allow_mentions
    }

    pub fn use_member_nicks(&self) -> bool {
        self.discord_config.use_member_nicks
    }

    pub fn webhook_enabled(&self) -> bool {
        self.discord_config.webhook_config.enabled
    }

    pub fn webhook_url(&self) -> String {
        self.discord_config.webhook_config.url.clone()
    }

    pub fn get_rcon_addr(&self) -> String {
        format!(
            "{}:{}",
            self.minecraft_config.rcon_ip, self.minecraft_config.rcon_port
        )
    }

    pub fn get_rcon_password(&self) -> String {
        self.minecraft_config.rcon_password.clone()
    }

    pub fn get_death_keywords(&self) -> Vec<String> {
        self.minecraft_config.custom_death_keywords.clone()
    }

    pub fn get_log_path(&self) -> String {
        self.minecraft_config.log_file_path.clone()
    }

    pub fn get_attachment_template(&self) -> String {
        self.minecraft_config.templates.attachment_template.clone()
    }

    pub fn get_message_template(&self) -> String {
        self.minecraft_config.templates.message_template.clone()
    }

    pub fn get_username_template(&self) -> String {
        self.minecraft_config.templates.username_template.clone()
    }

    pub fn use_listener(&self) -> bool {
        self.listener_config.enabled
    }

    pub fn get_listener_port(&self) -> u16 {
        self.listener_config.port
    }
}
