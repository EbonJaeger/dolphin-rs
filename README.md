# Dolphin-RS

This is an implementation of my [Dolphin](https://gitlab.com/EbonJaeger/dolphin) Discord/Minecraft bridge in Rust, in order to help me learn Rust. This project is the successor and continuation of Dolphin.

## Building and Running

### Building

You'll need Cargo to build this. If you don't have Cargo installed, you can get it via rustup [here](https://rustup.rs).

To build the release (and optimized) version, run `cargo build --release`. The resulting binary will be `target/release/dolphin-rs`. To build and run `dolphin-rs` all in one go, you can use `cargo run --release`.

### Precompiled

You should be able to just run the attached precompiled binary found on the [releases page](https://github.com/EbonJaeger/dolphin-rs/releases) without anything extra.

## Setup

Create a Discord bot [here](https://discord.com/developers/applications/me).

In the Bot tab on the left, you **must** enable the following Privilaged Intents:

- Presense Intent
- Server Members Intent
- Message Content Intent

These are required so Minecraft players can tag Discord users, and so the bot can read Discord messages to send them to the Minecraft server.

To start the bot, it has to know the Bot Token, shown on your bot's Discord page. It also needs the Application ID to create the command interactions; this is found on the bot's General Information page. Dolphin uses environment variables named `DISCORD_TOKEN` and `DISCORD_APPLICATION_ID` for this. You can set this variable automatically; look up guides on how to do this for your particular operating system. Or, you could set it when you run the Dolphin program, typically: `DISCORD_TOKEN=<paste the token here> DISCORD_APPLICATION_ID=<paste ID here> ./dolphin-rs`

Next, invite the bot to your Discord server using this link, replacing the Client ID with your bot's ID:

```
https://discord.com/api/oauth2/authorize?client_id=<CLIENT_ID>&permissions=10240&scope=bot
```

In your Minecraft server.properties, set the following options and restart the server:

```
enable-rcon=true
rcon.password=<password>
rcon.port=<1-65535>
```

Place the downloaded or built binary where ever you want, and run it to generate the config. By default, the config is generated and looked for in `$HOME/.config/dolphin/dolphin.toml` on macOS/Linux or `C:\Users\<you>\AppData\Local\dolphin\dolphin.toml` on Windows. The config can also be edited via Discord commands. Type `!help` in Discord for more.

### Using Discord Webhooks

Using a Discord webhook allows for much nicer messages to the Discord channel from Minecraft, such as using a different avatar for each Minecraft user and each message using their name. 

Minecraft avatars are provided via the [Crafatar API](https://crafatar.com).

Setting it up is easy:

1. In Discord, go to your server settings, go to Webhooks, and create a new webhook for the channel you wish to use.

2. Copy the Webhook URL shown, and paste it in your Dolphin config, and enable using webhooks. Start Dolphin and that's it, you're done! :D

### Listening for Remote Messages

If you want to use this with a Minecraft server that is not on the same machine, you can enable the webserver listener in the config to listen for `POST` messages on the configured TCP port at the `/message` endpoint. For an easy way to send these messages, check out [dolphin-send](https://github.com/EbonJaeger/dolphin-send). If you wish to do this yourself, `dolphin-rs` expects the messages to have a body of content type `application/json` with this JSON schema:

```json
{
  "name": "Username",
  "content": "The message you want to send to the Discord channel.",
  "source": "Player",
  "uuid": "Mojang UUID for fetching avatars"
}
```

`source` must be either `"Server"` or `"Player"`, and the name may be an empty string for non-player messages.

### Minecraft Message Template

You can customize the message format for messages being sent to Minecraft (via the [tellraw command](https://minecraft.gamepedia.com/Commands/tellraw)). For a list of the various things you can use with the tellraw command, see [this wiki page](https://minecraft.gamepedia.com/Raw_JSON_text_format#Java_Edition). If you are unsure about what this does, the defaults match Vanilla Minecraft chat output.

#### Defaults

`username_template`:

```json
{
  "color": "white",
  "text": "<%username%> ",
  "clickEvent": { "action": "suggest_command", "value": "%mention% " }
}
```

`attachment_template`:

```json
{
  "color": "gray",
  "text": "[%num% attachment(s) sent]",
  "clickEvent": { "action": "open_url", "value": "%url%" },
  "hoverEvent": { "action": "show_text", "value": { "text": "Click to open" } }
}
```

`message_template`:

```json
{ "color": "white", "text": "%content%" }
```

#### Placeholders

There are a few placeholders you can use in the templates to customize your chat messages in Minecraft:

- `%content%`
- `%mention%`
- `%num%` **Note:** This is only used for attachment messages to show how many attachments there are.
- `%url%` **Note:** This is only used for attachment messages to open the first attachment on click.
- `%username%`

### Chat Regex

You can use your own pattern to match chat messages from your server in case you have a custom chat format via server plugins. The default setting matches vanilla chat messages.

The parser expects there to be two [named capture groups](https://docs.rs/regex/1.4.5/regex/#grouping-and-flags): `username` and `content`. You can view the entirety of Rust's Regex syntax [here](https://docs.rs/regex/1.4.5/regex/#syntax). Most of the syntax is the same as other Regex engines. It is recommended to use something like [Regexr](https://regexr.com) to help you form a pattern.

Default: `^<(?P<username>\w+)> (?P<content>.+)`

## Usage

```
./dolphin-rs [FLAGS] [OPTIONS]
```

Flags:

```
    --debug         - Print additional debug lines to stdout
-h  --help          - Print the help message
-V  --version       - Prints the version info
```

Options:

```
-c  --config <FILE> - Load or generate the config at the given path
```

## License

Copyright &copy; 2020-2021 Evan Maddock <maddock.evan@vivaldi.net>

Dolphin-RS is available under the terms of the Apache 2.0 license.
