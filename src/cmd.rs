use diesel::prelude::*;

use std::collections::HashMap;
use std::iter::Iterator;
use std::str::SplitWhitespace;

use account::*;
use shared::*;

/// Type for command functions
type CmdFn = fn(&mut SplitWhitespace) -> ConnAction;

#[derive(Debug)]
pub enum ConnAction {
    Disconnect,
    Login(String),
    Noop(String),
}

use self::ConnAction::*;

// Create a table for commands. This particular use case doesn't need to be a hashmap, but I wanted
// to see if it could be done for when I get around to letting players alias their own commands.
lazy_static! {
    static ref CONN_CMDS: HashMap<&'static str, CmdFn> = {
        let mut m = HashMap::new();
        m.insert("help", help as CmdFn);
        m.insert("quit", quit as CmdFn);
        m.insert("register", register as CmdFn);
        m.insert("login", login as CmdFn);
        //m.insert("stats", stats as CmdFn);
        //m.insert("who", who as CmdFn);
        m
    };
}

/// Display the splash text
fn help(_line: &mut SplitWhitespace) -> ConnAction {
    Noop(SPLASH.to_string())
}

/// Say goodbye to the player and disconnect them
fn quit(_line: &mut SplitWhitespace) -> ConnAction {
    Disconnect
}

// TODO this should auto-login the player and change their state
/// Attempt to register a new player account
fn register(line: &mut SplitWhitespace) -> ConnAction {
    if let Some(name) = line.next() {
        if let Some(passwd) = line.next() {
            match Account::new(name.to_string(), passwd.to_string()) {
                Ok(acct) => return Login(format!("Registered new user: {}", acct.name)),
                Err(e) => return Noop(e),
            }
        }
    }
    Noop(format!("Registration Failed\n"))
}

/// Log a player into their account
fn login(line: &mut SplitWhitespace) -> ConnAction {
    if let Some(name) = line.next() {
        if let Some(passwd) = line.next() {
            match Account::login(name.to_string(), passwd.to_string()) {
                Ok(acct) => return Login(format!("Successfully logged in as {}\n", acct.name)),
                Err(e) => return Noop(e),
            }
        }
    }
    Noop(format!("Invalid login. Try again.\n"))
}

/// Parse commands for players in `Connected` state
pub fn cmd_connected(input: String) -> ConnAction {
    let mut line = input.split_whitespace();
    if let Some(cmd) = line.next() {
        let cmd_match: Vec<&str> = CONN_CMDS
            .keys()
            .filter(|k| k.starts_with(cmd))
            .map(|&s| s)
            .collect();
        if cmd_match.is_empty() {
            let reline = format!("{} {}", cmd, line.collect::<String>());
            let mut reline = reline.split_whitespace();
            login(&mut reline)
        } else if cmd_match.len() > 1 {
            Noop(format!(
                "Ambiguous command: {:?}\nMatches:{:?}\n",
                cmd, cmd_match
            ))
        } else {
            let func = CONN_CMDS.get(cmd_match.first().unwrap()).unwrap();
            func(&mut line)
        }
    } else {
        Noop("".to_string())
    }
}
