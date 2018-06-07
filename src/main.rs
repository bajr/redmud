//  Copyright (C) 2018 Bradley Rasmussen
//  This program is free software provided under the terms of the GNU LGPLv3 or later.
//  Full license information may be found in the LICENSE file in this distribution.

// Disclosures:
//  Net code was templated from chat examples in Tokio; https://github.com/tokio-rs/tokio

extern crate argon2rs;
extern crate bytes;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rand;
extern crate simplelog;
extern crate tk_listen;
extern crate tokio;

use bytes::Bytes;
use futures::sync::mpsc;
use tk_listen::*;
use tokio::net::TcpListener;
use tokio::prelude::*;

use std::fs::OpenOptions;
use std::sync::Arc;
//use std::thread;
use std::time::Duration;

mod account;
mod cmd;
mod lines;
mod player;
mod schema;
mod shared;

use player::Player;
use shared::Shared;

// TODO Make these bounded
// Shorthand for the transmit/receive parts of the message channel.
type Tx = mpsc::UnboundedSender<Bytes>;
type Rx = mpsc::UnboundedReceiver<Bytes>;

pub fn main() {
    init_logger();
    // TODO read configs from file

    // FIXME Figure out how best to allow all thread to talk to the DB when they need to...
    let db_url = "postgres://redmud:redmud@localhost/redmuddb";
    let share = Arc::new(Shared::new(&db_url));

    let addr = "127.0.0.1:3389".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    // TODO Find a way to give this master thread some portion of the available memory
    //    let (master_tx, master_rx) = mpsc::unbounded();
    //    let master = thread::Builder::new()
    //        .name("master".to_string())
    //        .spawn(move || {});

    // The server task asynchronously iterates over and processes each incoming connection.
    let server = listener
        .incoming()
        .sleep_on_error(Duration::from_secs(1))
        .map(move |socket| {
            // Spawn a task to process the connection
            let player = Player::new(share.clone(), socket);
            let connection = player.map_err(|e| {
                error!("Connection error = {:?}", e);
            });
            tokio::spawn(connection);
            Ok(())
        })
        .listen(1000)
        .map_err(|err| {
            error!("Fatal socket error. {:?}", err);
        });

    info!("Server running on {}", addr);

    tokio::run(server);
}

// Set up logging utility for server monitoring and debugging info
fn init_logger() {
    let log_conf = simplelog::Config {
        time: Some(simplelog::Level::Error),
        level: Some(simplelog::Level::Error),
        target: Some(simplelog::Level::Error),
        location: Some(simplelog::Level::Debug),
        time_format: Some("[%F %T%.3f %Z]"),
    };
    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(simplelog::LevelFilter::Debug, log_conf).unwrap(),
        simplelog::WriteLogger::new(
            simplelog::LevelFilter::Info,
            log_conf,
            OpenOptions::new()
                .write(true)
                .create(true)
                .open("redmud.log")
                .expect("Could not open log file!"),
        ),
    ]).expect("Failed to initialize logger");
}
