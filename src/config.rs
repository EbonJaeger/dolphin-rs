extern crate confy;

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RootConfig {
    discord_config: DiscordConfig,
    minecraft_config: MinecraftConfig,
    webserver_config: WebserverConfig,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DiscordConfig {
    channel_id: u64,
    allow_mentions: bool,
    use_member_nicks: bool,
    webhook_url: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MinecraftConfig {
    rcon_ip: String,
    rcon_port: i32,
    rcon_password: String,
    custom_death_keywords: Vec<String>,
    log_file_path: String,
    chat_regex: String,
    templates: TellrawTemplates,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebserverConfig {
    enabled: bool,
    port: u16,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TellrawTemplates {
    username_template: String,
    attachment_template: String,
    message_template: String,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        DiscordConfig {
            channel_id: 0,
            allow_mentions: true,
            use_member_nicks: false,
            webhook_url: String::new(),
        }
    }
}

impl Default for MinecraftConfig {
    fn default() -> Self {
        MinecraftConfig {
            rcon_ip: String::from("localhost"),
            rcon_port: 25575,
            rcon_password: String::new(),
            custom_death_keywords: Vec::new(),
            log_file_path: String::new(),
            chat_regex: String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
            templates: TellrawTemplates::default(),
        }
    }
}

impl Default for TellrawTemplates {
    fn default() -> Self {
        TellrawTemplates {
            username_template: String::from("{\"color\": \"white\", \"text\": \"<%username%> \", \"clickEvent\":{\"action\":\"suggest_command\", \"value\":\"%mention% \"}}",),
            attachment_template: String::from("{\"color\":\"gray\",\"text\":\"[%num% attachment(s) sent]\", \"clickEvent\":{\"action\":\"open_url\",\"value\":\"%url%\"},\"hoverEvent\":{\"action\":\"show_text\",\"value\":{\"text\":\"Click to open\"}}}"),
            message_template: String::from("{\"color\":\"white\", \"text\":\"%content%\"}"),
        }
    }
}

impl Default for WebserverConfig {
    fn default() -> Self {
        WebserverConfig {
            enabled: false,
            port: 25585,
        }
    }
}

impl RootConfig {
    pub fn get_channel_id(&self) -> u64 {
        self.discord_config.channel_id
    }

    pub fn mentions_allowed(&self) -> bool {
        self.discord_config.allow_mentions
    }

    pub fn use_member_nicks(&self) -> bool {
        self.discord_config.use_member_nicks
    }

    pub fn webhook_url(&self) -> String {
        self.discord_config.webhook_url.clone()
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

    pub fn get_chat_regex(&self) -> String {
        self.minecraft_config.chat_regex.clone()
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

    pub fn enable_webserver(&self) -> bool {
        self.webserver_config.enabled
    }

    pub fn get_webserver_port(&self) -> u16 {
        self.webserver_config.port
    }

    pub fn set_discord_channel(&mut self, channel: u64) {
        self.discord_config.channel_id = channel;
    }

    pub fn set_allow_mentions(&mut self, value: bool) {
        self.discord_config.allow_mentions = value;
    }

    pub fn set_use_nicks(&mut self, value: bool) {
        self.discord_config.use_member_nicks = value;
    }

    pub fn set_rcon_addr(&mut self, value: String) {
        self.minecraft_config.rcon_ip = value;
    }

    pub fn set_rcon_port(&mut self, value: i32) {
        self.minecraft_config.rcon_port = value;
    }

    pub fn set_rcon_password(&mut self, value: String) {
        self.minecraft_config.rcon_password = value;
    }

    pub fn set_log_file(&mut self, value: String) {
        self.minecraft_config.log_file_path = value;
    }

    pub fn set_chat_regex(&mut self, value: String) {
        self.minecraft_config.chat_regex = value;
    }

    pub fn set_webhook_url(&mut self, value: String) {
        self.discord_config.webhook_url = value;
    }
}
