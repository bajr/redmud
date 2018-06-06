use diesel::insert_into;
use diesel::prelude::*;

use std::collections::HashMap;
use std::iter::Iterator;
use std::str::SplitWhitespace;
use std::sync::Arc;

use account::*;
use shared::*;

/// Type for command functions
type CmdFn = fn(Arc<Shared>, &mut SplitWhitespace) -> Option<String>;

#[derive(Debug)]
enum Action {
    Quit,
    Register,
    Noop,
}

// Create a table for commands. This particular use case doesn't need to be a hashmap, but I wanted
// to see if it could be done for when I get around to letting players alias their own commands.
lazy_static! {
    static ref CONN_CMDS: HashMap<&'static str, CmdFn> = {
        let mut m = HashMap::new();
        m.insert("help", help as CmdFn);
        m.insert("quit", quit as CmdFn);
        m.insert("register", register as CmdFn);
        //m.insert("stats", stats as CmdFn);
        //m.insert("who", who as CmdFn);
        m
    };
}

/// Display the splash text
fn help(_share: Arc<Shared>, _line: &mut SplitWhitespace) -> Option<String> {
    Some(SPLASH.to_string())
}

/// Say goodbye to the player and disconnect them
fn quit(_share: Arc<Shared>, _line: &mut SplitWhitespace) -> Option<String> {
    None
}

/// Attempt to register a new player account
fn register(share: Arc<Shared>, line: &mut SplitWhitespace) -> Option<String> {
    use schema::accounts;

    if let Some(name) = line.next() {
        if let Some(passwd) = line.next() {
            let acct = Account::new(name.to_string(), passwd.to_string());
            let db_conn = share.db_conn.lock().unwrap();
            insert_into(accounts::table)
                .values(&acct)
                .get_result(db_conn)
                .expect("Error creating user account");
            return Some(format!("I found a name: {}\n", name));
        } else {
            Some(format!("No password given\n"))
        }
    } else {
        Some(format!("No name given\n"))
    }
}

// Parse commands for players in Connected state
pub fn cmd_connected(share: Arc<Shared>, input: String) -> Option<String> {
    let mut line = input.split_whitespace();
    if let Some(cmd) = line.next() {
        let cmd_match: Vec<&str> = CONN_CMDS
            .keys()
            .filter(|k| k.starts_with(cmd))
            .map(|&s| s)
            .collect();
        if cmd_match.is_empty() {
            Some(format!("Unknown command: {:?}\n", cmd))
        } else if cmd_match.len() > 1 {
            Some(format!(
                "Ambiguous command: {:?}\nMatches:{:?}\n",
                cmd, cmd_match
            ))
        } else {
            info!("Processing: {:?}", cmd);
            let func = CONN_CMDS.get(cmd_match.first().unwrap()).unwrap();
            func(share, &mut line)
        }
    } else {
        info!("We didn't get anything: {}", input);
        Some("".to_string())
    }
}
