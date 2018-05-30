# RedMUD

A basic MUD server implementation in Rust using [Tokio](https://tokio.rs/) for handling network
connections and [Diesel](https://diesel.rs/) for backend data storage.

## Getting Started

RedMUD is in the initial stages of development and is not yet in a configurable state.

RedMUD requires a stable version of [Rust](https://www.rust-lang.org/en-US/), and an installation
of PostgreSQL 10 with a database named `redmud`. Future implementations of RedMUD will provide
installation scripts to minimize this manual dependency setup.

Once dependencies are met, run RedMUD with `cargo run`. The server will available via any MUD
client or telnet by pointing them to localhost on port 3389.

## Implementation Goals

Currently, all RedMUD can do is accept connections and display some very minimal information to
connected players.

Further work will hopefully allow RedMUD to do the following:

* Register new player accounts
* Associate player accounts with a character
* Allow players to interact with a generic world map defined in the database
* Allow players to interact with other players selectively via a variety of built-in commands

## Stretch Goals

TBD

## License

RedMUD is licensed under [LGPL 3.0+](LICENSE)

## Author

[Bradley Rasmussen](https://github.com/bajr): rasmbj@pdx.edu
