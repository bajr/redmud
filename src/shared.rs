use diesel::pg::PgConnection;
use diesel::prelude::*;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Mutex, RwLock};
use std::time::SystemTime;

use super::Tx;

// Splash text displayed to all new connections
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

// TODO I need to rethink this data structure and how it's shared.
// Specifically, the reasonability of sharing this data with every player connection vs having
// 'thin' threads for player connections pass messages to a master thread for processing.
// I can worry about that later though. This works for now.
/// A structure of all the things that need to be shared safely among all players
pub struct Shared {
    pub players: RwLock<HashMap<SocketAddr, Tx>>, // We will read this more often than write
    pub db_conn: Mutex<PgConnection>,             // DB access is a blocking operation
    srv_stats: Mutex<Stats>,
}

/// Fun server statistics
struct Stats {
    start_time: SystemTime,
    player_count: u32,
}

impl Stats {
    fn new() -> Self {
        Stats {
            start_time: SystemTime::now(),
            player_count: 0,
        }
    }
}

impl Shared {
    pub fn new(db_url: &str) -> Self {
        let conn =
            PgConnection::establish(db_url).expect(&format!("Error connecting to {}", db_url));
        Shared {
            players: RwLock::new(HashMap::new()),
            db_conn: Mutex::new(conn),
            srv_stats: Mutex::new(Stats::new()),
        }
    }
}
