use diesel::pg::PgConnection;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::SystemTime;

use super::Tx;
use player::Player;

// Splash text displayed to all new connections
pub static SPLASH: &str = "Welcome to RedMud. Please choose an option:\n\
                           \x20 h(elp)      - Display this menu again\n\
                           \x20 q(uit)      - Quit\n\
                           \x20 l(ogin)     - Register as a new player\n\
                           \x20 r(egister)  - Register as a new player\n\
                           \x20 s(tats)     - Display server status and information\n\
                           \x20 w(ho)       - List players logged in\n\
                           \n\
                           Or enter your username to log in.\n\
                           \n\
                           Your choice: ";

lazy_static! {
    pub static ref SHARE: Shared = {
        let db_url = "postgres://redmud:redmud@localhost/redmuddb";
        Shared::new(&db_url)
    };
}

// TODO I need to rethink this data structure and how/why it's shared.
// Specifically, the reasonability of sharing this data via a lazy_static vs having
// 'thin' threads for player connections pass messages to a master thread for processing.
// I can worry about that later though. This works for now.
/// A structure of all the things that need to be shared safely among all players
pub struct Shared {
    pub conn_players: Mutex<HashMap<SocketAddr, Tx>>,
    pub play_players: Mutex<HashMap<String, Tx>>,
    pub db_conn: Pool<ConnectionManager<PgConnection>>,
    srv_stats: Mutex<Stats>,
}

/// Fun server statistics
struct Stats {
    start_time: SystemTime,
    player_count: u32,
}

impl Shared {
    fn new(db_url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(db_url);
        let db_conn = Pool::builder()
            .build(manager)
            .expect("Failed to create database connection pool.");
        Shared {
            conn_players: Mutex::new(HashMap::new()),
            play_players: Mutex::new(HashMap::new()),
            db_conn,
            srv_stats: Mutex::new(Stats::new()),
        }
    }
}

impl Stats {
    fn new() -> Self {
        Stats {
            start_time: SystemTime::now(),
            player_count: 0,
        }
    }
}
