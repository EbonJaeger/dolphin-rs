use std::time::Duration;

use crate::ConfigContainer;
use fancy_regex::Regex;
use rcon::Connection;
use serenity::{
    model::application::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::*,
    utils::Colour,
};
use tokio::time::sleep;

pub async fn list(ctx: Context, command: ApplicationCommandInteraction) -> anyhow::Result<()> {
    let config = ctx
        .data
        .read()
        .await
        .get::<ConfigContainer>()
        .cloned()
        .expect("expected config container in TypeMap");

    // Create RCON connection
    let addr = config.read().await.get_rcon_addr();

    let mut conn = Connection::builder()
        .enable_minecraft_quirks(true)
        .connect(addr, config.read().await.get_rcon_password().as_str())
        .await?;

    // Send the `list` command to the Minecraft server
    let mut resp = conn.cmd("minecraft:list").await?;
    if resp.starts_with("Unknown or incomplete command") {
        resp = conn.cmd("list").await?;
    }

    send_reply(&ctx, command, resp).await
}

async fn send_reply(
    ctx: &Context,
    command: ApplicationCommandInteraction,
    resp: String,
) -> anyhow::Result<()> {
    // Parse the response
    let mut parts = resp.split(':');
    let count_line = parts.next().unwrap();
    let player_list = parts.next().unwrap_or("");

    let (online, max) = get_player_counts(count_line);

    // Respond to the interaction
    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| {
                    data.embed(|e| {
                        e.title("Online Players")
                            .description(format!(
                                "There are **{}** out of **{}** players online.",
                                online, max
                            ))
                            .color(Colour::BLUE);

                        if !player_list.is_empty() {
                            e.footer(|f| f.text(player_list));
                        }

                        e
                    })
                })
        })
        .await?;

    sleep(Duration::new(30, 0)).await;
    command
        .delete_original_interaction_response(&ctx.http)
        .await?;

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
