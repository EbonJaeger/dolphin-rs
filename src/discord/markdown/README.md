This is a stripped down version of the Rust crate `markdown` written by Johann Hofmann, licensed under the Apache 2.0 license. You can find the original work [here](https://github.com/johannhof/markdown.rs). The vast majority of the credit here goes to him.

# Differences

Since this application only deals with Discord's flavor of Markdown and Minecraft formatting, there is a lot of the original crate that isn't needed here. On top of that, there are a couple of formatting types that Discord uses that aren't present in the upstream library.

Only the following elements are implemented:

- Blockquotes
- Emphasis
- Strikethrough
- Strong
- Underline

# License

All work **except** the strikethrough and underline parsers, and the Minecraft format conversion code is &copy; Johann Hofmann.

The license for the original work can be found [here](https://github.com/johannhof/markdown.rs/blob/master/LICENSE-APACHE). I really make no claims on top of that.
