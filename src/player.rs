extern crate tokio;

use bytes::{Bytes, BytesMut};
use futures::sync::mpsc;
use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use lines::{RecvLines, SendLines};
use shared::*;

use super::Tx;

#[derive(Debug)]
enum State {
    Connected, // Player just connected and has not yet logged in
    Idle,      // Player is logged in but not in the game world
    Playing,   // Player is playing
    Prison,    // Player is being punished
}

// The state for each connected client
pub struct Player {
    whoall: Arc<Mutex<Shared>>, // Every player has a handle to the list of connected players
    insock: RecvLines,          // Socket through which we will receive input from the player
    outsock: SendLines,         // Socket through which we will send outpu to the player
    addr: SocketAddr,           // The addr is saved so that the Drop impl can clean up its entry
    state: State,               // Player's activity state. Are they logged in?
}
//account: Account,          // A player account may have multiple characters

// TODO Bind players to their Account after they've connected
impl Player {
    pub fn new(whoall: Arc<Mutex<Shared>>, sock: TcpStream) -> Player {
        // Get the client socket address
        let addr = sock.peer_addr().unwrap();

        // Create a channel for this peer
        let (tx, rx) = mpsc::unbounded();

        // Split the socket so we can process input and output separately
        let (recv, send) = sock.split();
        let insock = RecvLines::new(recv);
        let outsock = SendLines::new(send, rx);

        // Add this player to the list.
        whoall.lock().unwrap().players.insert(addr, tx);

        Player {
            insock,
            outsock,
            whoall,
            addr,
            state: State::Connected,
        }
    }

    fn process_input(&mut self, input: &[u8]) -> Option<()> {
        let line = String::from_utf8(input.to_vec()).unwrap();
        let list = &self.whoall.lock().unwrap().players;
        let selftx = list.get(&self.addr)?;
        // Process player input based on their current state
        match self.state {
            State::Connected => {
                // Match input to menu options or a log-in attempt
                match line.as_str() {
                    "h" => {
                        selftx.unbounded_send(Bytes::from(SPLASH)).unwrap();
                        self.outsock.poll();
                    }
                    "q" => {
                        selftx
                            .unbounded_send(Bytes::from(&b"Thanks for playing!\n"[..]))
                            .unwrap();
                        self.outsock.poll();
                        return None;
                    }
                    "r" => unimplemented!(),
                    "s" => unimplemented!(),
                    "w" => unimplemented!(),
                    _ => unimplemented!(),
                }
                // If log-in unsuccessful, display splash screen again
                // If log-in successful, link them to the Account put them in Idle state
            }
            State::Idle => {
                // If they are enterring the world, put them into Playing state
                // If they logout, set Account to None and put them in Connected state
                unimplemented!();
            }
            State::Playing => {
                // process input
                unimplemented!();
            }
            State::Prison => {
                unimplemented!();
                // process input
            }
        }

        // Now, send the line to all other peers (except ourselves).
        //        for (addr, tx) in &self.whoall.lock().unwrap().players {
        //            if *addr != self.addr {
        //                tx.unbounded_send(line.clone()).unwrap();
        //            }
        //        }
        Some(())
    }
}

// A `Player` is also a future. When the socket closes, the future completes.
//
// While processing, the peer future implementation will:
// 1) Receive messages on its message channel and write them to the socket.
// 2) Receive messages from the socket and broadcast them to all peers.
impl Future for Player {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        // I need a better way of sending data to the receiver, this only updates them when they
        // send something to us.
        let _ = self.outsock.poll();
        // Read new lines from the socket
        while let Async::Ready(line) = self.insock.poll()? {
            info!("{:?}", self.state);
            if let Some(message) = line {
                match self.process_input(&message) {
                    None => return Ok(Async::Ready(())),
                    _ => {}
                };
            } else {
                // EOF was reached, remote client has disconnected, nothing more to do
                return Ok(Async::Ready(()));
            }
        }

        Ok(Async::NotReady)
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.whoall.lock().unwrap().players.remove(&self.addr);
    }
}
