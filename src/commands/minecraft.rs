use crate::{commands::embed::*, ConfigContainer};
use rcon::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};
use tokio::time::{delay_for, Duration};

#[command]
#[description = "List all online players on the Minecraft server."]
pub async fn list(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(config) = data.get::<ConfigContainer>() {
        // Create RCON connection
        let addr = format!(
            "{}:{}",
            config.minecraft_config.rcon_ip, config.minecraft_config.rcon_port
        );

        let mut conn = match Connection::builder()
            .enable_minecraft_quirks(true)
            .connect(addr, config.minecraft_config.rcon_password.as_str())
            .await
        {
            Ok(conn) => conn,
            Err(e) => {
                error!("Error performing list command: {:?}", e);
                send_error_embed(ctx, msg, "Error performing list command!", e).await?;
                return Ok(());
            }
        };

        // Send the `list` command to the Minecraft server
        let resp = match conn.cmd("minecraft:list").await {
            Ok(resp) => {
                if resp.starts_with("Unknown or incomplete command") {
                    conn.cmd("list").await.unwrap()
                } else {
                    resp
                }
            }
            Err(e) => {
                error!("Error performing list command: {:?}", e);
                send_error_embed(ctx, msg, "Error performing list command!", e).await?;
                return Ok(());
            }
        };

        send_reply(ctx, msg, resp).await?;
    } else {
        send_error_embed(ctx, msg, "Error performing list command!", "unknown error").await?;
    }

    Ok(())
}

async fn send_reply(ctx: &Context, msg: &Message, resp: String) -> CommandResult {
    // Parse the response
    let mut parts = resp.split(':');
    let count_line = parts.next().unwrap();
    let player_list = parts.next().unwrap();

    if let Some((online, max)) = get_player_counts(count_line) {
        // Create and send the embed
        let reply = match msg
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
                });

                m
            })
            .await
        {
            Ok(message) => message,
            Err(e) => {
                error!("Error sending command reply: {:?}", e);
                return Ok(());
            }
        };

        delay_for(Duration::new(30, 0)).await;
        reply.delete(&ctx.http).await?;
        msg.delete(&ctx.http).await?;
    }

    Ok(())
}

fn get_player_counts(text: &str) -> Option<(i32, i32)> {
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

    Some((online, max))
}
