# Dolphin-RS

This is an implementation of my Dolphin Discord/Minecraft bridge in Rust, in order to help me learn Rust. This project may or may not ever actually go anywhere.

## Building and Running

To build the release (and optimized) version, run `cargo build --release`. The resulting binary will be `target/release/dolphin-rs`.

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
