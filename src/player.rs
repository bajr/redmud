extern crate tokio;

use bytes::Bytes;
use futures::sync::mpsc;
use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

use std::net::SocketAddr;
use std::sync::Arc;

use cmd::*;
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
    share: Arc<Shared>, // Handle to various 'global' structures
    insock: RecvLines,  // Socket through which we will receive input from the player
    outsock: SendLines, // Socket through which we will send outpu to the player
    addr: SocketAddr,   // The addr is saved so that the Drop impl can clean up its entry
    state: State,       // Player's activity state. Are they logged in?
    tx: Tx,
}
//account: Account,          // A player account may have multiple characters

// TODO Bind players to their Account after they've connected
impl Player {
    pub fn new(share: Arc<Shared>, sock: TcpStream) -> Player {
        // Get the client socket address
        let addr = sock.peer_addr().unwrap();

        // Create a channel for this peer
        let (tx, rx) = mpsc::unbounded();

        // Split the socket so we can process input and output separately
        let (recv, send) = sock.split();
        let insock = RecvLines::new(recv);
        let outsock = SendLines::new(send, rx);

        // Add this player to the list.
        share.players.lock().unwrap().insert(addr, tx.clone());

        Player {
            share,
            insock,
            outsock,
            addr,
            state: State::Connected,
            tx,
        }
    }

    // TODO Does it make sense to separate command parsing from command processing? Why?
    // Parse player's input and process any valid commands for their current connection state
    fn process_input(&mut self, input: &[u8]) -> Option<String> {
        let mut line = String::from_utf8_lossy(input).into_owned();
        line.retain(|c| !c.is_control());
        // Process player input based on their current state
        let action = match self.state {
            State::Connected => cmd_connected(self.share.clone(), line),
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
        };
        action
    }
}

// A `Player` is also a future. When the socket closes, the future completes.
impl Future for Player {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        // I need a better way of sending data to the receiver, this only updates them when they
        // send something to us.
        let _ = self.outsock.poll();
        // Read new lines from the socket
        while let Async::Ready(line) = self.insock.poll()? {
            if let Some(message) = line {
                match self.process_input(&message) {
                    None => {
                        let _ = self.tx
                            .unbounded_send(Bytes::from(&b"Thanks for playing!\n"[..]));
                        let _ = self.outsock.poll();
                        return Ok(Async::Ready(()));
                    }
                    Some(msg) => {
                        let _ = self.tx.unbounded_send(Bytes::from(msg));
                    }
                };
            } else {
                // EOF was reached, client has disconnected
                return Ok(Async::Ready(()));
            }
            let _ = self.outsock.poll();
        }

        Ok(Async::NotReady)
    }
}

// This is called when a player disconnects in order to remove them from the shared player list.
impl Drop for Player {
    fn drop(&mut self) {
        debug!("Player Disconnected");
        self.share.players.lock().unwrap().remove(&self.addr);
    }
}
