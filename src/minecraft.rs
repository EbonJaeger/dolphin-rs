use std::collections::HashMap;

use fancy_regex::Regex;
use serde::Deserialize;

#[derive(Clone)]
pub struct MessageParser {
    cached_uuids: HashMap<String, String>,
    death_keywords: Vec<String>,
}

impl MessageParser {
    /// Create a new MessageParser to parse Minecraft log lines.
    pub fn new(mut custom_keywords: Vec<String>) -> Self {
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
        ];

        death_keywords.append(&mut custom_keywords);

        Self {
            cached_uuids: HashMap::new(),
            death_keywords,
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
        ];

        let mut cached_uuids = HashMap::new();
        cached_uuids.insert(
            String::from("EbonJaeger"),
            String::from("7f7c909b-24f1-49a4-817f-baa4f4973980"),
        );

        Self {
            cached_uuids,
            death_keywords,
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
    pub fn parse_line(&mut self, line: &str, regex: String) -> Option<MinecraftMessage> {
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
            &self
                .cached_uuids
                .insert(String::from(name), String::from(uuid));
            return None;
        }

        let chat_regex = Regex::new(&regex).unwrap();

        // Check if the line is a chat message
        if chat_regex.is_match(&line).unwrap() {
            let captures = chat_regex
                .captures(line)
                .expect("line matched, but couldn't get captures")
                .expect("line matched, but captures not found");

            // Use pattern matching to get the username and content
            // of the message
            match captures.name("username") {
                Some(name) => match captures.name("content") {
                    Some(content) => {
                        // Get the player's UUID so we can get their skin later
                        let uuid = match self.cached_uuids.get(name.as_str()) {
                            Some(uuid) => uuid.to_string(),
                            None => String::from("MHF_Steve"),
                        };

                        Some(MinecraftMessage {
                            name: name.as_str().to_string(),
                            content: content.as_str().to_string(),
                            source: Source::Player,
                            uuid,
                        })
                    }
                    None => None,
                },
                None => None,
            }
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
            // Check if the line is a player death message
            for word in &self.death_keywords {
                if line.contains(word.as_str())
                    && line != "Found that the dragon has been killed in this world already."
                {
                    return Some(MinecraftMessage {
                        name: String::new(),
                        content: format!(":skull: {}", line),
                        source: Source::Server,
                        uuid: String::new(),
                    });
                }
            }

            None
        }
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

#[cfg(test)]
mod tests {
    use crate::minecraft::MessageParser;
    use crate::minecraft::MinecraftMessage;
    use crate::minecraft::Source;

    #[test]
    fn parse_vanilla_chat_line() {
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
        match parser.parse_line(
            &input,
            String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
        ) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse chat message"),
        }
    }

    #[test]
    fn parse_non_vanilla_chat_line() {
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
        match parser.parse_line(
            &input,
            String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
        ) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse non-vanilla chat message"),
        }
    }

    #[test]
    fn parse_custom_chat_line() {
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
        match parser.parse_line(&input, String::from(r"(?P<username>\w+): (?P<content>.+)$")) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse non-vanilla chat message"),
        }
    }

    #[test]
    fn parse_join_line() {
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
        match parser.parse_line(
            &input,
            String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
        ) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse join message"),
        }
    }

    #[test]
    fn parse_leave_line() {
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
        match parser.parse_line(
            &input,
            String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
        ) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse leave message"),
        }

        if parser.cached_uuids().contains_key("EbonJaeger") {
            panic!("UUID cache still contains username after leave");
        }
    }

    #[test]
    fn parse_advancement_line() {
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
        match parser.parse_line(
            &input,
            String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
        ) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse advancement message"),
        }
    }

    #[test]
    fn parse_advancement2_line() {
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
        match parser.parse_line(
            &input,
            String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
        ) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse challenge message"),
        }
    }

    #[test]
    fn parse_server_start_line() {
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
        match parser.parse_line(
            &input,
            String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
        ) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse server started message"),
        }
    }

    #[test]
    fn parse_server_stop_line() {
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
        match parser.parse_line(
            &input,
            String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
        ) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse server stopped message"),
        }
    }

    #[test]
    fn parser_ignore_villager_death_message() {
        // Given
        let input = String::from("[12:32:45] [Server thread/INFO]: Villager axw['Villager'/85, l='world', x=-147.30, y=57.00, z=-190.70] died, message: 'Villager was squished too much'");
        let mut parser = MessageParser::new_for_test();

        // When/Then
        if let Some(_) = parser.parse_line(
            &input,
            String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
        ) {
            panic!("parsed a message when the line should be ignored")
        }
    }

    #[test]
    fn parser_cache_uuid_on_join() {
        // Given
        let input = String::from(
            "[19:54:56] [User Authenticator #1/INFO]: UUID of player EbonJaeger is 7f7c909b-24f1-49a4-817f-baa4f4973980",
        );
        let mut parser = MessageParser::new_for_test();

        // When
        if let None = parser.parse_line(
            &input,
            String::from(r"^<(?P<username>\w+)> (?P<content>.+)"),
        ) {
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
