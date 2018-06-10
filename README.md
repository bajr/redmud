# RedMUD

A basic MUD server implementation in Rust using [Tokio](https://tokio.rs/) for handling network
connections and [Diesel](https://diesel.rs/) for backend data storage.

## Getting Started

RedMUD is in the initial stages of development and is not yet in a configurable state. The
following instructions are written for Linux, but other systems should be possible.

1. RedMUD requires a stable version of [Rust](https://www.rust-lang.org/en-US/), PostgreSQL 10 and
[`diesel_cli`](https://github.com/diesel-rs/diesel/tree/master/diesel_cli).

1. Create a database cluster and log into the database as the `postgres` superuser:

    sudo pg_createcluster 10 main --start
    sudo -u postgres psql postgres

1. Create an administrative role for the RedMUD database (choosing a secure password).
We will use this role to initialize the database:

    CREATE ROLE redmud_admin LOGIN CREATEROLE CREATEDB PASSWORD 'wordpass';
    GRANT pg_monitor TO redmud_admin;

1. Use diesel to initialize the database (substituting your password):

    diesel setup --database-url='postgres://redmud_admin:wordpass@localhost/redmuddb'

1. Once all dependencies are installed and the database is initialized, run RedMUD with `cargo run`.
The server will available via any MUD client or telnet by pointing them to localhost on port 3389.

## Implementation Goals

Currently, all RedMUD can do is accept connections and display some very minimal information to
connected players.

Further work will hopefully allow RedMUD to do the following:

* Associate player accounts with a character
* Allow players to interact with a generic world map defined in the database
* Allow players to interact with other players selectively via a variety of built-in commands

## Stretch Goals

TBD

## License

RedMUD is licensed under [LGPL 3.0+](LICENSE)

## Author

[Bradley Rasmussen](https://github.com/bajr): rasmbj@pdx.edu
