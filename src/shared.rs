use std::collections::HashMap;
use std::net::SocketAddr;

use super::Tx;

pub static SPLASH: &str = "Welcome to RedMud. Please choose an option:\n\
                           \x20 h - Display this menu again\n\
                           \x20 q - Quit\n\
                           \x20 r - Register as a new player\n\
                           \x20 s - Display server status and information\n\
                           \x20 w - List players logged in\n\
                           \n\
                           Or enter your username to log in.\n\
                           \n\
                           Your choice: ";

// I need to rethink this data structure and how it's shared.
// This is a shared list of `Tx` handles for all connected clients.
pub struct Shared {
    pub players: HashMap<SocketAddr, Tx>,
}

impl Shared {
    pub fn new() -> Self {
        Shared {
            players: HashMap::new(),
        }
    }
}
