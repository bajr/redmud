//  Copyright (C) 2018 Bradley Rasmussen
//  This program is free software provided under the terms of the GNU LGPLv3 or later.
//  Full license information may be found in the LICENSE file in this distribution.

// Resource disclosures:
//  Net code was templated from examples provided by the tokio project; https://github.com/tokio-rs/tokio
//  This project is loosely modeled after the TinyMush project; git@github.com:TinyMUSH/TinyMUSH.git
extern crate diesel;
extern crate futures;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate tokio;

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

use std::collections::HashMap;
use std::iter;
use std::env;
use std::io::BufReader;
use std::fs::{File, OpenOptions};
use std::sync::{Arc, Mutex};

// Plan of attack
//    1. Set up network connection and loop that I can telnet into
//    1.5 Splash screen
//    2. Receive input
//    3. Translate input to commands
//    4. Set up player authentication
//    5. Set up Data structures for... Players, Characters, Maps/Rooms, Items

// Stretch goals
//    TLS/SSL
//    IPv6
//    IP Blacklisting
//    Separate logging for server events and in-world events
//    Admin interface? In-game? External?
//    Leave as much as possible outside of the code. Load a config file that lets the end user

// Other considerations
//    Command parsing
//    Permissions
//    Modules
//    Threading
//    Data persistance, consistency
//    Netcode management
//    Logging
//    Data structures: for players, characters, maps, entities, objects
//    Configuration parsing
//    Output: UTF-8?, Colors?
//    Resource limits: name lengths, # connections, command length

// Initialize Mud state, 'variables'.
// Log levels
// Externally configurable names/defaults
// Potential states???
// Set up memory for various buffers
// ??? DB conversions for various MUD data standards?
// Parse options (debug, restart, mindb, convert, help)
// Read config file via argument or default
// Ignore input on stdin?
// Initialize:...
// Dynamic linking libraries
// time trackers, caches, log files
// configurations, resource limits, commands
// Set up network

fn main() {
    init_logger();
    let addr = "127.0.0.1:4201".parse().unwrap();
    let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

    info!("Listening on: {}", addr);

    // This is running on the Tokio runtime, so it will be multi-threaded. The
    // `Arc<Mutex<...>>` allows state to be shared across the threads.
    let connections = Arc::new(Mutex::new(HashMap::new()));

    // The server task asynchronously iterates over each incoming connection.
    let srv = listener
        .incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |stream| {
            // The client's socket address
            let addr = stream.peer_addr().unwrap();

            println!("New Connection: {}", addr);

            // Split the TcpStream into two separate handles. One for reading and one for writing.
            let (reader, writer) = stream.split();

            // Create a channel for our stream, which other sockets will use to send us messages.
            // Then register our address with the stream to send data to us.
            let (tx, rx) = futures::sync::mpsc::unbounded();
            connections.lock().unwrap().insert(addr, tx);

            // Define here what we do for the actual I/O. That is, read a bunch of lines from the
            // socket and dispatch them while we also write any lines from other sockets.
            let connections_inner = connections.clone();
            let reader = BufReader::new(reader);

            // Model the read portion of this socket by mapping an infinite iterator to each line
            // off the socket. This "loop" is then terminated with an error once we hit EOF on the socket.
            let iter = stream::iter_ok::<_, io::Error>(iter::repeat(()));

            let socket_reader = iter.fold(reader, move |reader, _| {
                // Read a line off the socket, failing if we're at EOF
                let line = io::read_until(reader, b'\n', Vec::new());
                let line = line.and_then(|(reader, vec)| {
                    if vec.len() == 0 {
                        Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))
                    } else {
                        Ok((reader, vec))
                    }
                });

                // Convert the bytes we read into a string, and then send that string to all other connected clients.
                let line = line.map(|(reader, vec)| (reader, String::from_utf8(vec)));

                // Move the connection state into the closure below.
                let connections = connections_inner.clone();

                line.map(move |(reader, message)| {
                    info!("{}: {:?}", addr, message);
                    let mut conns = connections.lock().unwrap();

                    if let Ok(msg) = message {
                        // For each open connection except the sender, send the string via the channel.
                        let iter = conns
                            .iter_mut()
                            .filter(|&(&k, _)| k != addr)
                            .map(|(_, v)| v);
                        for tx in iter {
                            tx.unbounded_send(format!("{}: {}", addr, msg)).unwrap();
                        }
                    } else {
                        let tx = conns.get_mut(&addr).unwrap();
                        tx.unbounded_send("You didn't send valid UTF-8.".to_string())
                            .unwrap();
                    }

                    reader
                })
            });

            // Whenever we receive a string on the Receiver, we write it to `WriteHalf<TcpStream>`.
            let socket_writer = rx.fold(writer, |writer, msg| {
                let amt = io::write_all(writer, msg.into_bytes());
                let amt = amt.map(|(writer, _)| writer);
                amt.map_err(|_| ())
            });

            // Now that we've got futures representing each half of the socket, we use the `select`
            // combinator to wait for either half to be done to tear down the other. Then we spawn
            // off the result.
            let connections = connections.clone();
            let socket_reader = socket_reader.map_err(|_| ());
            let connection = socket_reader.map(|_| ()).select(socket_writer.map(|_| ()));

            // Spawn a task to process the connection
            tokio::spawn(connection.then(move |_| {
                connections.lock().unwrap().remove(&addr);
                println!("Connection {} closed.", addr);
                Ok(())
            }));

            Ok(())
        });

    // execute server
    tokio::run(srv);
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

// Initialize memory pools for various buffers
// Set up umask and figure out version
// If we are called with the name 'dbconvert', do a DB conversion and exit
// Parse CLI options
// Get config file from arguments or use default
/* Make sure we can read the config file */
// Throw usage error if we got an unrecognized option or couldn't read the config file
// Ignore all input on stdin: https://stackoverflow.com/questions/288062/is-close-fclose-on-stdin-guaranteed-to-be-correct
// initialize libltdl
// Init start time, restart, time, time since cpu counter reset
// Init Trace cache, player cache
// Init mudconf
// Init resource limits
// Init hash tables for: Commands, flags, powers, functions, attributes, etc
// Write startup stuff to log
// Read & validate config file
// Figure out if server is already running
// Set up help & status files
// Init user defined attributes
// Load modules
// Open database or start a new one and validate
// Start the DNS and identd lookup slave process
// Resize hash tables after startups are run, in order to get a really good idea of what's actually out there.
// Broadcast startup message
// Start timer and begin the big loop.
//init_timer();
//shovechars(mudconf.port);
// Shut stuff down
