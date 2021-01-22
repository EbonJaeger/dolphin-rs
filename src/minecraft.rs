use serde::Deserialize;

#[derive(Clone)]
pub struct MessageParser {
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

        Self { death_keywords }
    }

    ///
    /// Parse a line from a log file. If it is a message that we
    /// want to send over to Discord, it will return a `MinecraftMessage`.
    /// If the line does not match anything we want, `None` will be returned.
    ///
    pub fn parse_line(&self, line: &str) -> Option<MinecraftMessage> {
        let line = match trim_prefix(line) {
            Some(line) => line.trim(),
            None => return None,
        };

        // Ignore villager death messages
        if line.starts_with("Villager") && line.contains("died, message:") {
            return None;
        }

        // Check if the line is a chat message
        if line.starts_with('<') {
            parse_chat_line(line)
        } else if line.contains("joined the game") || line.contains("left the game") {
            // Join/leave message
            Some(MinecraftMessage {
                name: String::new(),
                content: String::from(line),
                source: Source::Server,
            })
        } else if is_advancement(line) {
            // Player Advancement message
            Some(MinecraftMessage {
                name: String::new(),
                content: format!(":partying_face: {}", line),
                source: Source::Server,
            })
        } else if line.starts_with("Done (") {
            // Server started message
            Some(MinecraftMessage {
                name: String::new(),
                content: String::from(":white_check_mark: Server has started"),
                source: Source::Server,
            })
        } else if line.starts_with("Stopping the server") {
            // Server stopping message
            Some(MinecraftMessage {
                name: String::new(),
                content: String::from(":x: Server is shutting down"),
                source: Source::Server,
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

fn parse_chat_line(line: &str) -> Option<MinecraftMessage> {
    // Split the message into parts
    let parts = line.splitn(2, ' ').collect::<Vec<&str>>();

    // Trim the < and > from the username part of the line
    let name = match parts[0].get(1..parts[0].len() - 1) {
        Some(username) => username,
        None => return None,
    };

    let message = parts[1];

    Some(MinecraftMessage {
        name: String::from(name),
        content: String::from(message),
        source: Source::Player,
    })
}

/// Trims the timestamp and thread prefix from incoming messages
/// from the Minecraft server. We have to check for multiple prefixes because
/// different server softwares change logging output slightly.
///
/// Returns None if the line doesn't contain an expected prefix.
fn trim_prefix(line: &str) -> Option<&str> {
    // Some server plugins may log abnormal lines
    if !line.starts_with('[') || line.len() < 11 {
        return None;
    }

    // Trim the timestamp prefix
    let trimmed = match line.get(11..) {
        Some(line) => line,
        None => return None,
    };

    // Return the line without the server thread prefix
    if trimmed.contains("[Server thread/INFO]: ") {
        trimmed.get(22..)
    } else if trimmed.contains("[Async Chat Thread") {
        trimmed.get(31..)
    } else {
        None
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum Source {
    Player,
    Server,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct MinecraftMessage {
    pub name: String,
    pub content: String,
    pub source: Source,
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
            String::from("[12:32:45] [Server thread/INFO]: <TestUser> Sending a chat message");
        let parser = MessageParser::new(vec![]);
        let expected = MinecraftMessage {
            name: String::from("TestUser"),
            content: String::from("Sending a chat message"),
            source: Source::Player,
        };

        // When/Then
        match parser.parse_line(&input) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse chat message"),
        }
    }

    #[test]
    fn parse_non_vanilla_chat_line() {
        // Given
        let input = String::from(
            "[12:32:45] [Async Chat Thread - #0/INFO]: <TestUser> Sending a chat message",
        );
        let parser = MessageParser::new(vec![]);
        let expected = MinecraftMessage {
            name: String::from("TestUser"),
            content: String::from("Sending a chat message"),
            source: Source::Player,
        };

        // When/Then
        match parser.parse_line(&input) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse non-vanilla chat message"),
        }
    }

    #[test]
    fn parse_join_line() {
        // Given
        let input = String::from("[12:32:45] [Server thread/INFO]: TestUser joined the game");
        let parser = MessageParser::new(vec![]);
        let expected = MinecraftMessage {
            name: String::new(),
            content: String::from("TestUser joined the game"),
            source: Source::Server,
        };

        // When/Then
        match parser.parse_line(&input) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse join message"),
        }
    }

    #[test]
    fn parse_leave_line() {
        // Given
        let input = String::from("[12:32:45] [Server thread/INFO]: TestUser left the game");
        let parser = MessageParser::new(vec![]);
        let expected = MinecraftMessage {
            name: String::new(),
            content: String::from("TestUser left the game"),
            source: Source::Server,
        };

        // When/Then
        match parser.parse_line(&input) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse leave message"),
        }
    }

    #[test]
    fn parse_advancement_line() {
        // Given
        let input = String::from(
            "[12:32:45] [Server thread/INFO]: TestUser has made the advancement [MonsterHunter]",
        );
        let parser = MessageParser::new(vec![]);
        let expected = MinecraftMessage {
            name: String::new(),
            content: String::from(
                ":partying_face: TestUser has made the advancement [MonsterHunter]",
            ),
            source: Source::Server,
        };

        // When/Then
        match parser.parse_line(&input) {
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
        let parser = MessageParser::new(vec![]);
        let expected = MinecraftMessage {
            name: String::new(),
            content: String::from(
                ":partying_face: TestUser has completed the challenge [MonsterHunter]",
            ),
            source: Source::Server,
        };

        // When/Then
        match parser.parse_line(&input) {
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
        let parser = MessageParser::new(vec![]);
        let expected = MinecraftMessage {
            name: String::new(),
            content: String::from(":white_check_mark: Server has started"),
            source: Source::Server,
        };

        // When/Then
        match parser.parse_line(&input) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse server started message"),
        }
    }

    #[test]
    fn parse_server_stop_line() {
        // Given
        let input = String::from("[12:32:45] [Server thread/INFO]: Stopping the server");
        let parser = MessageParser::new(vec![]);
        let expected = MinecraftMessage {
            name: String::new(),
            content: String::from(":x: Server is shutting down"),
            source: Source::Server,
        };

        // When/Then
        match parser.parse_line(&input) {
            Some(msg) => assert_eq!(msg, expected),
            None => panic!("failed to parse server stopped message"),
        }
    }

    #[test]
    fn parser_ignore_villager_death_message() {
        // Given
        let input = String::from("[12:32:45] [Server thread/INFO]: Villager axw['Villager'/85, l='world', x=-147.30, y=57.00, z=-190.70] died, message: 'Villager was squished too much'");
        let parser = MessageParser::new(vec![]);

        // When/Then
        if let Some(_) = parser.parse_line(&input) {
            panic!("parsed a message when the line should be ignored")
        }
    }
}
