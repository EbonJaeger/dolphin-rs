use linemux::MuxedLines;
use std::io::Error;
use tokio::stream::StreamExt;

pub struct MinecraftWatcher {
    death_keywords: Vec<String>,
    line_watcher: MuxedLines,
}

impl MinecraftWatcher {
    /// Create a new MinecraftWatcher with the location of the log file to
    /// continuously tail for messages.
    pub async fn new(
        mut custom_death_keywords: Vec<String>,
        log_location: String,
    ) -> Result<Self, Error> {
        let mut line_watcher = MuxedLines::new()?;
        line_watcher.add_file(&log_location).await?;
        debug!("Added log file to tail: {}", &log_location);

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

        death_keywords.append(&mut custom_death_keywords);

        Ok(Self {
            death_keywords,
            line_watcher,
        })
    }

    pub async fn read_line(&mut self) -> Option<MinecraftMessage> {
        let line_watcher = &mut self.line_watcher;
        // Read the next line from the log
        let line = match line_watcher.next().await {
            Some(line) => line,
            None => return None,
        };

        // Unbox the line Result into a Line
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Error reading a line from the Minecraft log: {}", e);
                return None;
            }
        };

        debug!("Received a line from Minecraft");

        // Shadow the variable with the contents of the line
        let line = line.line();

        // Parse the line and return a MinecraftMessage if it matches
        // something we're looking for
        self.parse_line(line)
    }

    fn parse_line(&self, line: &str) -> Option<MinecraftMessage> {
        let line = match self.trim_prefix(line) {
            Some(line) => line,
            None => return None,
        };

        let line = line.trim();

        // Ignore villager death messages
        if line.starts_with("Villager") && line.contains("died, message:") {
            return None;
        }

        // Check if the line is a chat message
        if line.starts_with("<") {
            self.parse_chat_line(line)
        } else if line.contains("joined the game") || line.contains("left the game") {
            // Join/leave message
            Some(MinecraftMessage {
                name: String::new(),
                message: String::from(line),
            })
        } else if self.is_advancement(line) {
            // Player Advancement message
            Some(MinecraftMessage {
                name: String::new(),
                message: String::from(format!(":partying_face: {}", line)),
            })
        } else if line.starts_with("Done (") {
            // Server started message
            Some(MinecraftMessage {
                name: String::new(),
                message: String::from(":white_check_mark: Server has started"),
            })
        } else if line.starts_with("Stopping the server") {
            // Server stopping message
            Some(MinecraftMessage {
                name: String::new(),
                message: String::from(":x: Server is shutting down"),
            })
        } else {
            // Check if the line is a player death message
            for word in &self.death_keywords {
                if line.contains(word)
                    && line != "Found that the dragon has been killed in this world already."
                {
                    return Some(MinecraftMessage {
                        name: String::new(),
                        message: String::from(format!(":skull: {}", line)),
                    });
                }
            }

            None
        }
    }

    /// Check if the line is the server logging a player earning
    /// an Advancement.
    fn is_advancement(&self, line: &str) -> bool {
        line.contains("has made the advancement")
            || line.contains("has completed the challenge")
            || line.contains("has reached the goal")
    }

    fn parse_chat_line(&self, line: &str) -> Option<MinecraftMessage> {
        // Split the message into parts
        let parts = line.splitn(2, " ");
        let parts = parts.collect::<Vec<&str>>();

        // Trim the < and > from the username part of the line
        let name = match parts[0].get(1..parts[0].len() - 2) {
            Some(username) => username,
            None => return None,
        };

        let message = parts[1];

        Some(MinecraftMessage {
            name: String::from(name),
            message: String::from(message),
        })
    }

    /// Trims the timestamp and thread prefix from incoming messages
    /// from the Minecraft server. We have to check for multiple prefixes because
    /// different server softwares change logging output slightly.
    ///
    /// Returns None if the line doesn't contain an expected prefix.
    fn trim_prefix<'a>(&self, line: &'a str) -> Option<&'a str> {
        // Some server plugins may log abnormal lines
        if !line.starts_with("[") || line.len() < 11 {
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
}

#[derive(Debug)]
pub struct MinecraftMessage {
    pub name: String,
    pub message: String,
}
