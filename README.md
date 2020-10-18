# Dolphin-RS

This is an implementation of my Dolphin Discord/Minecraft bridge in Rust, in order to help me learn Rust. This project may or may not ever actually go anywhere.

## Building and Running

### Building

You'll need Cargo to build this. If you don't have Cargo installed, you can get it via rustup [here](https://rustup.rs).

To build the release (and optimized) version, run `cargo build --release`. The resulting binary will be `target/release/dolphin-rs`. To build and run `dolphin-rs` all in one go, you can use `cargo run --release`.

### Precompiled

You should be able to just run the attached precompiled binary found on the [releases page](https://github.com/EbonJaeger/dolphin-rs/releases) without anything extra.

## Setup

Create a Discord bot [here](https://discordapp.com/developers/applications/me). Next, add the bot to your Discord server using this link, replacing the Client ID with your bot's ID:

```
https://discord.com/api/oauth2/authorize?client_id=<CLIENT_ID>&permissions=10240&scope=bot
```

In your Minecraft server.properties, set the following options and restart the server:

```
enable-rcon=true
rcon.password=<password>
rcon.port=<1-65535>
```

Place the downloaded or built binary where ever you want, and run it to generate the config. By default, the config is generated and looked for in `$HOME/.config/dolphin/dolphin.toml`.

### Using Discord Webhooks

Using a Discord webhook allows for much nicer messages to the Discord channel from Minecraft, such as using a different avatar for each Minecraft user and each message using their name. Setting it up is easy:

1. In Discord, go to your server settings, go to Webhooks, and create a new webhook for the channel you wish to use.

2. Copy the Webhook URL shown, and paste it in your Dolphin config, and enable using webhooks. Start Dolphin and that's it, you're done! :D

### Minecraft Message Template

You can (to a degree) customize the message format for messages being sent to Minecraft (via the [tellraw command](https://minecraft.gamepedia.com/Commands/tellraw)). This value should be a JSON array of objects. For a list of various things you can use, see [this wiki page](https://minecraft.gamepedia.com/Raw_JSON_text_format#Java_Edition). If you are unsure about what this does, the default matches Vanilla Minecraft chat output.

Default:

```json
[{"color": "white", "text": "<%username%> ", "clickEvent":{"action":"suggest_command", "value":"%mention% "}}, %content%]
```

### Placeholders

There are two placeholders that **must** be used: `%username%` and `%content%`, else the text wont be replaced and you'll end up with the placeholders in your chat window, and nobody wants that. Additionally you can use the following optional placeholders: `%mention%`

The way this is handled should be improved in the future, but that may lead to less customizability. I'm not sure how I want to handle that just yet.

## Usage

```
./dolphin-rs [FLAGS]
```

Flags:

```
    --debug   - Print additional debug lines to stdout
-h  --help    - Print the help message
-v  --version - Prints the version info
```

## License

Copyright &copy; 2020 Evan Maddock <maddock.evan@vivaldi.net>

Dolphin-RS is available under the terms of the Apache 2.0 license.
