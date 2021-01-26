# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add a converter to show Discord formatting in Minecraft

## [v2.2.0] - 2021-01-23

### Changed

- Update dependencies
  - Tokio runtime updated to 1.0
  - Serenity updated to 0.10
    - Allows handling of inline replies
    - Uses the new Discord v8 API
- Don't lock global data for the entirety of each command task
- Use inline replies for command responses

## [v2.1.1] - 2020-12-11

### Fixed

- Fix full username mentions with discriminator not being parsed correctly

## [v2.1.0] - 2020-12-03

### Added

- Add ability to mention roles and channels from Minecraft
- Add an optional webserver implementation to listen for messages from other machines

### Fixed

- Log files being moved (such as maybe during log rotation) should no longer break the bot, if that was happening
- Fix mentions from Minecraft with spaces not creating a mention

### Changed

- Print nicer-looking error messages

## [v2.0.1] - 2020-11-22

### Fixed

- Fix a bad value in the default configuration

### Changed

- Eliminated a call to the Discord REST API when messages are received from Minecraft
- Replace ugly Discord mentions with names in messages to Minecraft
- Escape double quote characters in messages to Minecraft

## [v2.0.0] - 2020-11-8

### Added

- Add more customization options for chat formatting in Minecraft

### Changed

- Improve experience when a user sends an attachment in Discord

[unreleased]: https://github.com/EbonJaeger/dolphin-rs/compare/v2.2.0...master
[v2.2.0]: https://github.com/EbonJaeger/dolphin-rs/compare/v2.1.1...v2.2.0
[v2.1.1]: https://github.com/EbonJaeger/dolphin-rs/compare/v2.1.0...v2.1.1
[v2.1.0]: https://github.com/EbonJaeger/dolphin-rs/compare/v2.0.1...v2.1.0
[v2.0.1]: https://github.com/EbonJaeger/dolphin-rs/compare/v2.0.0...v2.0.1
[v2.0.0]: https://github.com/EbonJaeger/dolphin-rs/compare/94a867f...v2.0.0