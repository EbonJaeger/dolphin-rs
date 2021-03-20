use std::time::Duration;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, id::ChannelId},
    prelude::Mentionable,
    utils::Colour,
};
use tokio::time::sleep;

use crate::{config::RootConfig, ConfigContainer, ConfigPathContainer};

use super::embed::send_error_embed;

#[command]
#[sub_commands(channel, mentions, nicks, rconaddr, rconport, rconpass, log)]
#[required_permissions("ADMINISTRATOR")]
pub async fn config(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let reply = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Configuration")
            .description("You can use commands to change various configuration options for Dolphin. For more information about these commands, type `!help config`")
            .color(Colour::BLUE);

            e
        })
        .reference_message(msg);

        m
    }).await?;

    // Wait 30 seconds and delete the command and reply
    sleep(Duration::new(30, 0)).await;
    msg.delete(&ctx).await?;
    reply.delete(&ctx).await?;

    Ok(())
}

#[command]
#[description = "Set which channel to use for sending Minecraft messages"]
pub async fn channel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let config_lock = {
        let config_read = ctx.data.read().await;

        config_read
            .get::<ConfigContainer>()
            .cloned()
            .expect("expected config container in TypeMap")
            .clone()
    };

    let channel = args.single::<ChannelId>()?;

    // Update the config inside a block so we release locks as soon as possible
    {
        let mut c = config_lock.write().await;
        c.set_discord_channel(channel.0);
        save_config(ctx, c.clone()).await?;
    }

    // Send a message letting the user know that the config updated
    let reply = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Configuration Changed")
                    .description(format!("Discord channel changed to {}", channel.mention()))
                    .color(Colour::DARK_GREEN);

                e
            })
            .reference_message(msg);

            m
        })
        .await?;

    // Wait 30 seconds and delete the command and reply
    sleep(Duration::new(30, 0)).await;
    reply.delete(&ctx).await?;
    msg.delete(&ctx).await?;

    Ok(())
}

#[command]
#[description = "Set if Minecraft players can mention Discord users"]
pub async fn mentions(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let config_lock = {
        let config_read = ctx.data.read().await;

        config_read
            .get::<ConfigContainer>()
            .cloned()
            .expect("expected config container in TypeMap")
            .clone()
    };

    let allow_mentions = args.single::<bool>()?;

    // Update the config inside a block so we release locks as soon as possible
    {
        let mut c = config_lock.write().await;
        c.set_allow_mentions(allow_mentions);
        save_config(ctx, c.clone()).await?;
    }

    // Send a message letting the user know that the config updated
    let reply = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Configuration Changed")
                    .description(format!("Allow mentions changed to `{}`", allow_mentions))
                    .color(Colour::DARK_GREEN);

                e
            })
            .reference_message(msg);

            m
        })
        .await?;

    // Wait 30 seconds and delete the command and reply
    sleep(Duration::new(30, 0)).await;
    reply.delete(&ctx).await?;
    msg.delete(&ctx).await?;

    Ok(())
}

#[command]
#[description = "Set if the bot should use Discord server nicknames when sending to Minecraft"]
pub async fn nicks(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let config_lock = {
        let config_read = ctx.data.read().await;

        config_read
            .get::<ConfigContainer>()
            .cloned()
            .expect("expected config container in TypeMap")
            .clone()
    };

    let use_nicks = args.single::<bool>()?;

    // Update the config inside a block so we release locks as soon as possible
    {
        let mut c = config_lock.write().await;
        c.set_use_nicks(use_nicks);
        save_config(ctx, c.clone()).await?;
    }

    // Send a message letting the user know that the config updated
    let reply = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Configuration Changed")
                    .description(format!("Use member nicks changed to `{}`", use_nicks))
                    .color(Colour::DARK_GREEN);

                e
            })
            .reference_message(msg);

            m
        })
        .await?;

    // Wait 30 seconds and delete the command and reply
    sleep(Duration::new(30, 0)).await;
    reply.delete(&ctx).await?;
    msg.delete(&ctx).await?;

    Ok(())
}

#[command]
#[description = "Set the Minecraft RCON address"]
pub async fn rconaddr(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let config_lock = {
        let config_read = ctx.data.read().await;

        config_read
            .get::<ConfigContainer>()
            .cloned()
            .expect("expected config container in TypeMap")
            .clone()
    };

    let addr = args.single::<String>()?;

    // Update the config inside a block so we release locks as soon as possible
    {
        let mut c = config_lock.write().await;
        c.set_rcon_addr(addr.clone());
        save_config(ctx, c.clone()).await?;
    }

    // Send a message letting the user know that the config updated
    let reply = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Configuration Changed")
                    .description(format!("Rcon address changed to `{}`", addr))
                    .color(Colour::DARK_GREEN);

                e
            })
            .reference_message(msg);

            m
        })
        .await?;

    // Wait 30 seconds and delete the command and reply
    sleep(Duration::new(30, 0)).await;
    reply.delete(&ctx).await?;
    msg.delete(&ctx).await?;

    Ok(())
}

#[command]
#[description = "Set the Minecraft RCON port"]
pub async fn rconport(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let config_lock = {
        let config_read = ctx.data.read().await;

        config_read
            .get::<ConfigContainer>()
            .cloned()
            .expect("expected config container in TypeMap")
            .clone()
    };

    let port = args.single::<i32>()?;

    if port > 65535 || port < 1024 {
        send_error_embed(
            &ctx,
            &msg,
            format!("Port '{}' not in range 1024-65535", port).as_str(),
            "E_INVALID_PORT",
        )
        .await?;
        return Ok(());
    }

    // Update the config inside a block so we release locks as soon as possible
    {
        let mut c = config_lock.write().await;
        c.set_rcon_port(port);
        save_config(ctx, c.clone()).await?;
    }

    // Send a message letting the user know that the config updated
    let reply = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Configuration Changed")
                    .description(format!("Rcon port changed to `{}`", port))
                    .color(Colour::DARK_GREEN);

                e
            })
            .reference_message(msg);

            m
        })
        .await?;

    // Wait 30 seconds and delete the command and reply
    sleep(Duration::new(30, 0)).await;
    reply.delete(&ctx).await?;
    msg.delete(&ctx).await?;

    Ok(())
}

#[command]
#[description = "Set the Minecraft RCON password"]
pub async fn rconpass(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Delete the message immedietly because it contains a password
    msg.delete(&ctx).await?;

    let config_lock = {
        let config_read = ctx.data.read().await;

        config_read
            .get::<ConfigContainer>()
            .cloned()
            .expect("expected config container in TypeMap")
            .clone()
    };

    let password = args.single::<String>()?;

    // Update the config inside a block so we release locks as soon as possible
    {
        let mut c = config_lock.write().await;
        c.set_rcon_password(password.clone());
        save_config(ctx, c.clone()).await?;
    }

    // Send a message letting the user know that the config updated
    let reply = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Configuration Changed")
                    .description("Rcon password updated")
                    .color(Colour::DARK_GREEN);

                e
            });

            m
        })
        .await?;

    // Wait 30 seconds and delete the reply
    sleep(Duration::new(30, 0)).await;
    reply.delete(&ctx).await?;

    Ok(())
}

#[command]
#[description = "Set the path to the Minecraft log file"]
pub async fn log(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let config_lock = {
        let config_read = ctx.data.read().await;

        config_read
            .get::<ConfigContainer>()
            .cloned()
            .expect("expected config container in TypeMap")
            .clone()
    };

    let path = args.single::<String>()?;

    // Update the config inside a block so we release locks as soon as possible
    {
        let mut c = config_lock.write().await;
        c.set_log_file(path.clone());
        save_config(ctx, c.clone()).await?;
    }

    // Send a message letting the user know that the config updated
    let reply = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Configuration Changed")
                    .description(format!("Minecraft log file changed to `{}`", path))
                    .color(Colour::DARK_GREEN);

                e
            })
            .reference_message(msg);

            m
        })
        .await?;

    // Wait 30 seconds and delete the command and reply
    sleep(Duration::new(30, 0)).await;
    reply.delete(&ctx).await?;
    msg.delete(&ctx).await?;

    Ok(())
}

/// Save the given configuration to the file on disk.
async fn save_config(ctx: &Context, config: RootConfig) -> CommandResult {
    // Get the path to the config if it isn't the default location
    let config_path = {
        let config_path = ctx.data.read().await;

        config_path
            .get::<ConfigPathContainer>()
            .cloned()
            .expect("expected config path container in TypeMap")
            .clone()
    };

    // Save the config to disk
    {
        if config_path.is_empty() {
            confy::store("dolphin", config)?;
        } else {
            confy::store_path(config_path.as_ref(), config)?;
        }
    }

    Ok(())
}
