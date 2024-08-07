use std::{collections::HashMap, sync::Arc};

use fancy_regex::Regex;
use serde::Deserialize;
use serenity::{
    model::prelude::GuildId,
    prelude::{Context, Mentionable},
};
use thiserror::Error;
use tracing::error;

#[derive(Clone)]
pub struct MessageParser {
    cached_uuids: HashMap<String, String>,
    death_keywords: Vec<String>,
    ignore_phrases: Vec<String>,
}

impl MessageParser {
    /// Create a new MessageParser to parse Minecraft log lines.
    pub fn new(mut custom_keywords: Vec<String>, mut ignore_keywords: Vec<String>) -> Self {
        let mut death_keywords = vec![
            String::from(" shot"),
            String::from(" pricked"),
            String::from(" walked into a cactus"),
            String::from(" roasted"),
            String::from(" drowned"),
            String::from(" kinetic"),
            String::from(" blew up"),
            String::from(" blown up"),
            String::from(" killed"),
            String::from(" hit the ground"),
            String::from(" fell"),
            String::from(" doomed"),
            String::from(" squashed"),
            String::from(" magic"),
            String::from(" flames"),
            String::from(" burned"),
            String::from(" walked into fire"),
            String::from(" burnt"),
            String::from(" bang"),
            String::from(" tried to swim in lava"),
            String::from(" lightning"),
            String::from("floor was lava"),
            String::from("danger zone"),
            String::from(" slain"),
            String::from(" fireballed"),
            String::from(" stung"),
            String::from(" starved"),
            String::from(" suffocated"),
            String::from(" squished"),
            String::from(" poked"),
            String::from(" imapled"),
            String::from("didn't want to live"),
            String::from(" withered"),
            String::from(" pummeled"),
            String::from(" died"),
            String::from(" slain"),
            String::from(" obliterated"),
        ];

        death_keywords.append(&mut custom_keywords);

        let mut ignore_phrases = vec![String::from(
            "Found that the dragon has been killed in this world already.",
        )];

        ignore_phrases.append(&mut ignore_keywords);

        Self {
            cached_uuids: HashMap::new(),
            death_keywords,
            ignore_phrases,
        }
    }

    /// Constructor for testing with a pre-filled cache.
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        let death_keywords = vec![
            String::from(" shot"),
            String::from(" pricked"),
            String::from(" walked into a cactus"),
            String::from(" roasted"),
            String::from(" drowned"),
            String::from(" kinetic"),
            String::from(" blew up"),
            String::from(" blown up"),
            String::from(" killed"),
            String::from(" hit the ground"),
            String::from(" fell"),
            String::from(" doomed"),
            String::from(" squashed"),
            String::from(" magic"),
            String::from(" flames"),
            String::from(" burned"),
            String::from(" walked into fire"),
            String::from(" burnt"),
            String::from(" bang"),
            String::from(" tried to swim in lava"),
            String::from(" lightning"),
            String::from("floor was lava"),
            String::from("danger zone"),
            String::from(" slain"),
            String::from(" fireballed"),
            String::from(" stung"),
            String::from(" starved"),
            String::from(" suffocated"),
            String::from(" squished"),
            String::from(" poked"),
            String::from(" imapled"),
            String::from("didn't want to live"),
            String::from(" withered"),
            String::from(" pummeled"),
            String::from(" died"),
            String::from(" slain"),
            String::from(" obliterated"),
        ];

        let ignore_phrases = vec![String::from(
            "Found that the dragon has been killed in this world already.",
        )];

        let mut cached_uuids = HashMap::new();
        cached_uuids.insert(
            String::from("EbonJaeger"),
            String::from("7f7c909b-24f1-49a4-817f-baa4f4973980"),
        );

        Self {
            cached_uuids,
            death_keywords,
            ignore_phrases,
        }
    }

    /// Helper function for testing to inspect the username to UUID cache.
    ///
    /// The returned [HashMap] os a cloned version of the parser's `HashMap`.
    #[cfg(test)]
    pub fn cached_uuids(&self) -> HashMap<String, String> {
        self.cached_uuids.clone()
    }

    /// Parse a line from a log file. If it is a message that we
    /// want to send over to Discord, it will return a [MinecraftMessage].
    /// If the line does not match anything we want, [None] will be returned.
    pub async fn parse_line(&mut self, line: &str, regex: String) -> Option<MinecraftMessage> {
        let line = match trim_prefix(line) {
            Some(line) => line.trim(),
            None => return None,
        };

        // Ignore villager death messages
        if line.starts_with("Villager") && line.contains("died, message:") {
            return None;
        }

        // See if we can use this line to cache a player's UUID
        if line.starts_with("UUID of player") {
            let parts: Vec<&str> = line.split(' ').collect();
            let name = parts[3];
            let uuid = parts[5];
            let _ = &self
                .cached_uuids
                .insert(String::from(name), String::from(uuid));
            return None;
        }

        let chat_regex = Regex::new(&regex).unwrap();

        // Check if the line is a chat message
        if chat_regex.is_match(line).unwrap() {
            self.try_parse_chat(chat_regex, line).await
        } else if line.contains("joined the game") || line.contains("left the game") {
            if line.contains("left the game") {
                // Leave message, so remove this player from the cache
                if let Some(end) = line.find(' ') {
                    if let Some(name) = line.get(..end) {
                        self.cached_uuids.remove(name);
                    }
                }
            }

            // Join/leave message
            Some(MinecraftMessage {
                name: String::new(),
                content: String::from(line),
                source: Source::Server,
                uuid: String::new(),
            })
        } else if is_advancement(line) {
            // Player Advancement message
            Some(MinecraftMessage {
                name: String::new(),
                content: format!(":partying_face: {}", line),
                source: Source::Server,
                uuid: String::new(),
            })
        } else if line.starts_with("Done (") {
            // Server started message
            Some(MinecraftMessage {
                name: String::new(),
                content: String::from(":white_check_mark: Server has started"),
                source: Source::Server,
                uuid: String::new(),
            })
        } else if line.starts_with("Stopping the server") {
            // Server stopping message
            Some(MinecraftMessage {
                name: String::new(),
                content: String::from(":x: Server is shutting down"),
                source: Source::Server,
                uuid: String::new(),
            })
        } else {
            self.try_parse_death(line)
        }
    }

    /// Try to parse a line as a chat message.
    ///
    /// The line will be split into two parts: the username and
    /// the message itself.
    async fn try_parse_chat(&mut self, chat_regex: Regex, line: &str) -> Option<MinecraftMessage> {
        let captures = chat_regex
            .captures(line)
            .expect("line matched, but couldn't get captures")
            .expect("line matched, but captures not found");

        let name = captures
            .name("username")
            .expect("log message matched chat regex, but there's no username")
            .as_str();

        let content = captures
            .name("content")
            .expect("log message matched chat regex, but there's no content")
            .as_str();

        let uuid = match self.get_player_uuid(name).await {
            Ok(uuid) => uuid,
            Err(_e) => String::from("c06f8906-4c8a-4911-9c29-ea1dbd1aab82"),
        };

        Some(MinecraftMessage {
            name: name.to_string(),
            content: content.to_string(),
            source: Source::Player,
            uuid,
        })
    }

    /// Get the player's UUID so we can get their skin later
    /// If the player isn't in our cache, try to get their UUID
    /// from the Mojang API using their username. If that fails,
    /// fallback to a UUID to a Steve skin.
    async fn get_player_uuid(&mut self, name: &str) -> Result<String, Error> {
        match self.cached_uuids.get(name) {
            Some(uuid) => Ok(uuid.to_string()),
            // Not found in cache, reach out to Mojang
            None => match uuid_from_name(name.to_string()).await {
                Ok(resp) => {
                    // Cache the found UUID
                    let _ = &self.cached_uuids.insert(resp.name, resp.id.clone());
                    // Return the UUID
                    Ok(resp.id)
                }
                Err(e) => match e {
                    Error::Http(e) => Err(Error::Http(e)),
                    _ => Err(Error::UUIDNotFound(name.to_string())),
                },
            },
        }
    }

    /// Try to parse a death message from a log line.
    ///
    /// First, we will check if the line contains keywords that
    /// should cause the message to be ignored.
    ///
    /// If we get past that, check if the message contains keywords
    /// that are a part of death messages.
    fn try_parse_death(&mut self, line: &str) -> Option<MinecraftMessage> {
        for ignore_phrase in &self.ignore_phrases {
            if line.contains(ignore_phrase.as_str()) {
                return None;
            }
        }

        // Named entity deaths commonly get logged, and gets matched
        // by the death keywords. Let's try to split the message, and
        // still take the actual death message, for added fun.
        if line.starts_with("Named entity") {
            let mut parts = line.split(':');
            _ = parts.next()?;
            let actual_line = parts.next()?.trim_start();
            return self.try_parse_death(actual_line);
        }

        let mut message: Option<MinecraftMessage> = None;

        for word in &self.death_keywords {
            if !line.contains(word.as_str()) {
                continue;
            }

            message = Some(MinecraftMessage {
                name: String::new(),
                content: format!(":skull: {line}"),
                source: Source::Server,
                uuid: String::new(),
            });
        }

        message
    }
}

/// Check if the line is the server logging a player earning
/// an Advancement.
fn is_advancement(line: &str) -> bool {
    line.contains("has made the advancement")
        || line.contains("has completed the challenge")
        || line.contains("has reached the goal")
}

/// Trims the timestamp and thread prefix from incoming messages
/// from the Minecraft server.
///
/// Returns None if the line doesn't contain an expected prefix.
fn trim_prefix(line: &str) -> Option<&str> {
    // Some server plugins may log abnormal lines
    if !line.starts_with('[') || line.len() < 11 {
        return None;
    }

    match line.find("]: ") {
        Some(index) => line.get(index + 3..),
        None => None,
    }
}

/// The source of a message. This is expected to be either "Player" or "Server".
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum Source {
    Player,
    Server,
}

/// Represents a message from a Minecraft server, with any metadata that may be
/// associated with it.
///
/// The `uuid` field is for a player's UUID for use in fetching their player skin
/// for the avatar to be used when sending the message to Discord.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct MinecraftMessage {
    pub name: String,
    pub content: String,
    pub source: Source,
    pub uuid: String,
}

impl MinecraftMessage {
    /// Looks for instances of mentions in a message and attempts
    /// to replace that text with an actual Discord `@mention` (or
    /// `#channel` in the case of a channel).
    ///
    /// It tries to match names using the full name and, in the
    /// case of users, optionally their  descriptor. This works
    /// for names that have spaces in them, and really probably
    /// anything else.
    pub fn replace_mentions(
        &mut self,
        ctx: Arc<Context>,
        guild_id: Arc<GuildId>,
    ) -> Result<(), Error> {
        let guild = match ctx.cache.guild(*guild_id) {
            Some(guild) => guild,
            None => return Err(Error::NoGuild(*guild_id)),
        };

        let mut found_start = false;
        let mut start = 0;
        let mut end = 0;
        let mut replaced = self.content.clone();

        for (i, c) in self.content.char_indices() {
            if !found_start && (c == '@' || c == '#') {
                found_start = true;
                start = i;
            } else if found_start && c == '#' {
                end = i + 5;
            } else if found_start && c == ' ' {
                end = i;
            } else if found_start && replaced.len() == i + 1 {
                end = i + 1;
            }

            // Check to see if we have a mention
            if found_start && end > 0 {
                if let Some(mention) = replaced.get(start..end) {
                    let name = &mention[1..];
                    if let Some(member) = guild.member_named(name) {
                        replaced = replaced.replace(mention, &member.mention().to_string());
                    } else if let Some(role) = guild.role_by_name(name) {
                        replaced = replaced.replace(mention, &role.mention().to_string());
                    } else if let Some(id) = guild
                        .channels
                        .iter()
                        .find(|&(_, v)| v.name == name)
                        .map(|(k, _)| k)
                    {
                        if let Some(channel) = guild.channels.get(id) {
                            replaced = replaced.replace(mention, &channel.mention().to_string());
                        }
                    } else {
                        continue;
                    }

                    // If we got here, we found a mention, so reset everything
                    start = 0;
                    end = 0;
                    found_start = false;
                }
            }
        }

        self.content = replaced;
        Ok(())
    }
}

#[derive(Deserialize)]
struct IdResponse {
    name: String,
    id: String,
}

async fn uuid_from_name(name: String) -> Result<IdResponse, Error> {
    let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", name);
    let resp: IdResponse = reqwest::get(url).await?.json().await?;
    Ok(resp)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("no guild found for ID '{0}'")]
    NoGuild(GuildId),

    #[error("no UUID found for name '{0}'")]
    UUIDNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::MessageParser;
    use super::MinecraftMessage;
    use super::Source;

    #[tokio::test]
    async fn parse_vanilla_chat_line() {
        // Given
        let input =
            String::from("[12:32:45] [Server thread/INFO]: <EbonJaeger> Sending a chat message");
        let mut parser = MessageParser::new_for_test();
        let expected = MinecraftMessage {
            name: String::from("EbonJaeger"),
            content: String::from("Sending a chat message"),
            source: Source::Player,
            uuid: String::from("7f7c909b-24f1-49a4-817f-baa4f4973980"),
        };

        // When/Then
        match parser
            .parse_line(
                &input,
                String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
            )
            .await
        {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse chat message"),
        }
    }

    #[tokio::test]
    async fn parse_non_vanilla_chat_line() {
        // Given
        let input =
            String::from("[12:32:45] [Chat Thread - #0/INFO]: <EbonJaeger> Sending a chat message");
        let mut parser = MessageParser::new_for_test();
        let expected = MinecraftMessage {
            name: String::from("EbonJaeger"),
            content: String::from("Sending a chat message"),
            source: Source::Player,
            uuid: String::from("7f7c909b-24f1-49a4-817f-baa4f4973980"),
        };

        // When/Then
        match parser
            .parse_line(
                &input,
                String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
            )
            .await
        {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse non-vanilla chat message"),
        }
    }

    #[tokio::test]
    async fn parse_custom_chat_line() {
        // Given
        let input = String::from(
            "[12:32:45] [Chat Thread - #0/INFO]: [Survival] EbonJaeger: Sending a chat message",
        );
        let mut parser = MessageParser::new_for_test();
        let expected = MinecraftMessage {
            name: String::from("EbonJaeger"),
            content: String::from("Sending a chat message"),
            source: Source::Player,
            uuid: String::from("7f7c909b-24f1-49a4-817f-baa4f4973980"),
        };

        // When/Then
        match parser
            .parse_line(&input, String::from(r"(?P<username>\w+): (?P<content>.+)$"))
            .await
        {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse non-vanilla chat message"),
        }
    }

    #[tokio::test]
    async fn parse_join_line() {
        // Given
        let input = String::from("[12:32:45] [Server thread/INFO]: TestUser joined the game");
        let mut parser = MessageParser::new_for_test();
        let expected = MinecraftMessage {
            name: String::new(),
            content: String::from("TestUser joined the game"),
            source: Source::Server,
            uuid: String::new(),
        };

        // When/Then
        match parser
            .parse_line(
                &input,
                String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
            )
            .await
        {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse join message"),
        }
    }

    #[tokio::test]
    async fn parse_leave_line() {
        // Given
        let input = String::from("[12:32:45] [Server thread/INFO]: EbonJaeger left the game");
        let mut parser = MessageParser::new_for_test();
        let expected = MinecraftMessage {
            name: String::new(),
            content: String::from("EbonJaeger left the game"),
            source: Source::Server,
            uuid: String::new(),
        };

        // When/Then
        match parser
            .parse_line(
                &input,
                String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
            )
            .await
        {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse leave message"),
        }

        if parser.cached_uuids().contains_key("EbonJaeger") {
            panic!("UUID cache still contains username after leave");
        }
    }

    #[tokio::test]
    async fn parse_advancement_line() {
        // Given
        let input = String::from(
            "[12:32:45] [Server thread/INFO]: TestUser has made the advancement [MonsterHunter]",
        );
        let mut parser = MessageParser::new_for_test();
        let expected = MinecraftMessage {
            name: String::new(),
            content: String::from(
                ":partying_face: TestUser has made the advancement [MonsterHunter]",
            ),
            source: Source::Server,
            uuid: String::new(),
        };

        // When/Then
        match parser
            .parse_line(
                &input,
                String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
            )
            .await
        {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse advancement message"),
        }
    }

    #[tokio::test]
    async fn parse_advancement2_line() {
        // Given
        let input = String::from(
            "[12:32:45] [Server thread/INFO]: TestUser has completed the challenge [MonsterHunter]",
        );
        let mut parser = MessageParser::new_for_test();
        let expected = MinecraftMessage {
            name: String::new(),
            content: String::from(
                ":partying_face: TestUser has completed the challenge [MonsterHunter]",
            ),
            source: Source::Server,
            uuid: String::new(),
        };

        // When/Then
        match parser
            .parse_line(
                &input,
                String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
            )
            .await
        {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse challenge message"),
        }
    }

    #[tokio::test]
    async fn parse_server_start_line() {
        // Given
        let input = String::from(
            "[12:32:45] [Server thread/INFO]: Done (21.3242s)! For help, type \"help\"",
        );
        let mut parser = MessageParser::new_for_test();
        let expected = MinecraftMessage {
            name: String::new(),
            content: String::from(":white_check_mark: Server has started"),
            source: Source::Server,
            uuid: String::new(),
        };

        // When/Then
        match parser
            .parse_line(
                &input,
                String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
            )
            .await
        {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse server started message"),
        }
    }

    #[tokio::test]
    async fn parse_server_stop_line() {
        // Given
        let input = String::from("[12:32:45] [Server thread/INFO]: Stopping the server");
        let mut parser = MessageParser::new_for_test();
        let expected = MinecraftMessage {
            name: String::new(),
            content: String::from(":x: Server is shutting down"),
            source: Source::Server,
            uuid: String::new(),
        };

        // When/Then
        match parser
            .parse_line(
                &input,
                String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
            )
            .await
        {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse server stopped message"),
        }
    }

    #[tokio::test]
    async fn parser_ignore_villager_death_message() {
        // Given
        let input = String::from("[12:32:45] [Server thread/INFO]: Villager axw['Villager'/85, l='world', x=-147.30, y=57.00, z=-190.70] died, message: 'Villager was squished too much'");
        let mut parser = MessageParser::new_for_test();

        // When/Then
        if let Some(_) = parser
            .parse_line(
                &input,
                String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
            )
            .await
        {
            panic!("parsed a message when the line should be ignored")
        }
    }

    #[tokio::test]
    async fn parser_splits_named_entity_death() {
        // Given
        let input = String::from("[12:32:45] [Server thread/INFO]: Named entity WanderingWinemakerEntity['Bobbie'/396211, l='ServerLevel[world]', x=-4882.04, y=70.00, z=-9015.70] died: Bobbie was slain by Zombie");
        let mut parser = MessageParser::new_for_test();

        // When
        let message = parser
            .parse_line(
                &input,
                String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
            )
            .await
            .expect("A message should have been generated.");

        // Then
        if message.content != ":skull: Bobbie was slain by Zombie" {
            panic!("parsed a named entity death message, but the contents aren't as expected: {} vs {}", input, message.content);
        }
    }

    #[tokio::test]
    async fn parser_cache_uuid_on_join() {
        // Given
        let input = String::from(
            "[19:54:56] [User Authenticator #1/INFO]: UUID of player EbonJaeger is 7f7c909b-24f1-49a4-817f-baa4f4973980",
        );
        let mut parser = MessageParser::new_for_test();

        // When
        if let None = parser
            .parse_line(
                &input,
                String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
            )
            .await
        {
            // Then
            if let Some(uuid) = parser.cached_uuids().get("EbonJaeger") {
                if uuid != "7f7c909b-24f1-49a4-817f-baa4f4973980" {
                    panic!("UUID cache incorrect: expected '7f7c909b-24f1-49a4-817f-baa4f4973980', got '{}'", uuid);
                }
            } else {
                panic!("username not found in UUID cache");
            }
        }
    }
}
