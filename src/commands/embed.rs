use crate::errors::{Error, Result};
use serenity::{client::Context, model::prelude::Message, utils::Colour};
use std::fmt::Display;
use tokio::time::{sleep, Duration};

///
/// Send an embed as a reply to a command if there was
/// and error while performing the command.
///
/// Both the reply and the originating command message
/// will be deleted after 30 seconds, assuming the embed
/// was sent successfully.
///
pub async fn send_error_embed<T>(ctx: &Context, msg: &Message, desc: &str, reason: T) -> Result<()>
where
    T: Display,
{
    // Send the error embed
    let reply = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(Colour::RED)
                    .title("Command Error")
                    .description(format!(":no_entry: {}", desc))
                    .footer(|f| f.text(reason))
            })
            .reference_message(msg)
        })
        .await?;

    // Wait 30 seconds and delete the reply and the originating message
    sleep(Duration::new(30, 0)).await;
    reply.delete(&ctx.http).await?;
    match msg.delete(&ctx.http).await {
        Ok(()) => Ok(()),
        Err(e) => Err(Error::Discord(e)),
    }
}
