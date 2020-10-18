use serenity::{
    framework::standard::{
        help_commands, macros::help, Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::prelude::*,
    prelude::*,
};
use std::collections::HashSet;

#[help]
#[command_not_found_text = "Could not find command '{}'."]
#[embed_success_colour = "blue"]
#[individual_command_tip = "To get help with an individual command, pass its name as an argument to this command. E.g. `!help list`\n"]
#[lacking_role = "hide"]
#[lacking_permissions = "hide"]
#[lacking_ownership = "hide"]
#[lacking_conditions = "hide"]
#[max_levenshtein_distance(3)]
#[wrong_channel = "nothing"]
pub async fn show_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
    Ok(())
}
