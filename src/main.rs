//  Copyright (C) 2018 Bradley Rasmussen
//  This program is free software provided under the terms of the GNU LGPLv3 or later.
//  Full license information may be found in the LICENSE file in this distribution.

// Disclosures:
//  Net code was templated from various examples in Tokio; https://github.com/tokio-rs/tokio
//  This project was loosely modeled from TinyMush; git@github.com:TinyMUSH/TinyMUSH.git

// FUNKY PUTTY TELNET NEGOTIATION
// [2018-05-09 01:02:38.935 UTC] [INFO] redmud: `b"\xff\xfb\x1f\xff\xfb
// \xff\xfb\x18\xff\xfb'\xff\xfd\x01\xff\xfb\x03\xff\xfd\x03TEST"` connected
// Works fine when putty does passive negotiation.

// Plan of attack
//    Startup:
//      0. Set up logger, read config, open and validate DB,
//      1. Prep connection handler.
//    Data Structures:
//      * Commands, flags, functions?
//      * Players, Characters, Creatures, Stats/Attributes/Abilities
//      * Maps/Rooms
//      * Entities, Items
//    On Connection (connections require state)
//      1. Display welcome splash
//      2. Authenticate or register
//      3. Materialize PC
//    Event handling:
//      *
//    Admin
//      * Start/stop server
//      * Player control (kick, ban, whitelist, blacklist, throttle, etc)
//      * Content creation
//    Other/Stretch Goals
//      Restart server without dropping connections
//      Help Pages
//      TLS/SSL
//      IPv6
//      Separate logging for debugging, server events and in-world events
//      Admin interface? In-game? External?
//      Different map structures: graphs, flat grids, geodesic grids, 3d grids.
//        - Does it make sense for a mud to have smooth coordinate systems? How would that work?
//      Play-by-Email mechanics influencing game world
//      Output: UTF-8, Colors
//      Modules
//      Dynamic Permissions

extern crate bytes;
extern crate diesel;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate tk_listen;
extern crate tokio;

use bytes::Bytes;
use futures::sync::mpsc;
use tk_listen::*;
use tokio::net::TcpListener;
use tokio::prelude::*;

use std::fs::OpenOptions;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod lines;
mod player;
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

    // Create the shared state. This is how all the peers communicate.
    //
    // The server task will hold a handle to this. For every new client, the `state` handle is
    // cloned and passed into the task that processes the client connection.
    let state = Arc::new(Mutex::new(Shared::new()));

    let addr = "127.0.0.1:3389".parse().unwrap();

    let listener = TcpListener::bind(&addr).unwrap();

    // The server task asynchronously iterates over and processes each incoming connection.
    let server = listener
        .incoming()
        .sleep_on_error(Duration::from_secs(1))
        .map(move |socket| {
            // Spawn a task to process the connection
            //process(socket, state.clone());
            let player = Player::new(state.clone(), socket);
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
            simplelog::LevelFilter::Debug,
            log_conf,
            OpenOptions::new()
                .write(true)
                .create(true)
                .open("redmud.log")
                .expect("Could not open log file!"),
        ),
    ]).expect("Failed to initialize logger");
    error!("Error");
    warn!("Warning");
    info!("Info");
    debug!("Debug");
    trace!("Trace");
}
