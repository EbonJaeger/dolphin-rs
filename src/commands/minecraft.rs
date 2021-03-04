use crate::{errors::Error, ConfigContainer};
use rcon::Connection;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};
use tokio::time::{sleep, Duration};

#[command]
#[description = "List all online players on the Minecraft server."]
pub async fn list(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let config = ctx
        .data
        .read()
        .await
        .get::<ConfigContainer>()
        .cloned()
        .expect("expected config container in TypeMap");

    // Create RCON connection
    let addr = config.get_rcon_addr();

    let mut conn = Connection::builder()
        .enable_minecraft_quirks(true)
        .connect(addr, config.get_rcon_password().as_str())
        .await?;

    // Send the `list` command to the Minecraft server
    let mut resp = conn.cmd("minecraft:list").await?;
    if resp.starts_with("Unknown or incomplete command") {
        resp = conn.cmd("list").await?;
    }

    send_reply(ctx, msg, resp).await?;

    Ok(())
}

async fn send_reply(ctx: &Context, msg: &Message, resp: String) -> Result<(), Error> {
    // Parse the response
    let mut parts = resp.split(':');
    let count_line = parts.next().unwrap();
    let player_list = parts.next().unwrap();

    let (online, max) = get_player_counts(count_line);

    // Create and send the embed
    let reply = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
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
            .reference_message(msg);

            m
        })
        .await?;

    sleep(Duration::new(30, 0)).await;
    reply.delete(&ctx.http).await?;
    msg.delete(&ctx.http).await?;

    Ok(())
}

fn get_player_counts(text: &str) -> (i32, i32) {
    let parts = text.split_whitespace();
    let mut got_online = false;
    let mut online = -1;
    let mut max = -1;

    for part in parts {
        let num = match part.parse::<i32>() {
            Ok(num) => num,
            Err(_) => continue,
        };

        if got_online {
            max = num;
        } else {
            online = num;
            got_online = true;
        }
    }

    (online, max)
}
