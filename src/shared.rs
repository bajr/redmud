use diesel::pg::PgConnection;
use diesel::prelude::*;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Mutex;
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

// TODO I need to rethink this data structure and how/why it's shared.
// Specifically, the reasonability of sharing this data with every player connection vs having
// 'thin' threads for player connections pass messages to a master thread for processing.
// I can worry about that later though. This works for now.
/// A structure of all the things that need to be shared safely among all players
pub struct Shared {
    pub players: Mutex<HashMap<SocketAddr, Tx>>, // We will read this more often than write
    pub db_conn: Pool<ConnectionManager<PgConnection>>,
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
        let manager = ConnectionManager::<PgConnection>::new(db_url);
        let db_conn = Pool::builder()
            .build(manager)
            .expect("Failed to create database connection pool.");
        Shared {
            players: Mutex::new(HashMap::new()),
            db_conn,
            srv_stats: Mutex::new(Stats::new()),
        }
    }
}
