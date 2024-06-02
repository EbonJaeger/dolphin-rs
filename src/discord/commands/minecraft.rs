use std::time::Duration;

use crate::config::container::ConfigContainer;
use fancy_regex::Regex;
use rcon::Connection;
use serenity::{
    all::CommandInteraction,
    builder::{
        CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
    },
    model::Colour,
    prelude::*,
};
use thiserror::Error;
use tokio::time::sleep;

pub async fn list(ctx: Context, command: CommandInteraction) -> Result<(), Error> {
    let config = ctx
        .data
        .read()
        .await
        .get::<ConfigContainer>()
        .cloned()
        .expect("expected config container in TypeMap");

    // Create RCON connection
    let addr = config.read().await.get_rcon_addr();
    let password = config.read().await.get_rcon_password();

    let mut conn = Connection::builder()
        .enable_minecraft_quirks(true)
        .connect(addr, password.as_str())
        .await?;

    // Send the `list` command to the Minecraft server
    let mut resp = conn.cmd("minecraft:list").await?;
    if resp.starts_with("Unknown or incomplete command") {
        resp = conn.cmd("list").await?;
    }

    send_reply(&ctx, command, resp).await
}

async fn send_reply(ctx: &Context, command: CommandInteraction, resp: String) -> Result<(), Error> {
    // Parse the response
    let mut parts = resp.split(':');
    let count_line = parts.next().unwrap();
    let player_list = parts.next().unwrap_or("");

    let (online, max) = get_player_counts(count_line);

    // Respond to the interaction
    let embed = CreateEmbed::new()
        .title("Online Players")
        .description(format!(
            "There are **{}** out of **{}** players online.",
            online, max
        ))
        .color(Colour::BLUE)
        .footer(CreateEmbedFooter::new(player_list));

    let response = CreateInteractionResponseMessage::new().add_embed(embed);

    command
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await?;

    sleep(Duration::new(30, 0)).await;
    command.delete_response(&ctx.http).await?;

    Ok(())
}

fn get_player_counts(text: &str) -> (i32, i32) {
    lazy_static! {
        static ref COUNT_REGEX: Regex = Regex::new(r"(?P<online>\d+)\D+(?P<max>\d+)").unwrap();
    }

    match COUNT_REGEX.captures(text) {
        Ok(result) => match result {
            Some(captures) => {
                let online = captures
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse::<i32>()
                    .expect("could not parse match as a number");
                let max = captures
                    .get(2)
                    .unwrap()
                    .as_str()
                    .parse::<i32>()
                    .expect("could not parse match as a number");
                (online, max)
            }
            None => (-1, -1),
        },
        Err(_) => (-1, -1),
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("command error: {0}")]
    Discord(#[from] serenity::Error),

    #[error("rcon error: {0}")]
    Rcon(#[from] rcon::Error),
}
