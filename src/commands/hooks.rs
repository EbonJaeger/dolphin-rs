use crate::commands::embed::send_error_embed;
use serenity::{
    framework::standard::{macros::hook, CommandResult},
    model::channel::Message,
    prelude::Context,
};
use tracing::{debug, error};

#[hook]
pub async fn after(ctx: &Context, msg: &Message, command_name: &str, result: CommandResult) {
    match result {
        Ok(()) => debug!(
            "dispatch:after: Successfully ran the '{}' command",
            command_name
        ),
        Err(e) => {
            error!("Error performing the '{}' command: {}", command_name, e);
            if let Err(e) = send_error_embed(
                ctx,
                msg,
                format!("Error performing `{}` command!", command_name).as_str(),
                e,
            )
            .await
            {
                error!("Error replying with a command error: {:?}", e);
            };
        }
    }
}
