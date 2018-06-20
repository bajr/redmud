use std::collections::BTreeMap;
use std::iter::Iterator;
use std::str::SplitWhitespace;

use account::*;
use shared::*;

type ConnFn = fn(&mut SplitWhitespace) -> ConnAction;

#[derive(Debug)]
pub enum ConnAction {
    Disconnect,
    Login(Account, String),
    Noop(String),
}

pub use self::ConnAction::*;

/// Tables of recognized commands before login
lazy_static! {
    static ref CONN_CMDS: BTreeMap<&'static str, ConnFn> = {
        let mut m = BTreeMap::new();
        m.insert("help", help as ConnFn);
        m.insert("quit", quit as ConnFn);
        m.insert("register", register as ConnFn);
        m.insert("login", login as ConnFn);
        //m.insert("stats", stats as ConnFn);
        //m.insert("who", who as ConnFn);
        m
    };
}

/// Display a list of currently logged in players
fn who(_line: &mut SplitWhitespace) -> ConnAction {
    //SHARE.players.lock().unwrap()
    Noop(format!("Not yet implemented\n"))
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
                Ok(acct) => return Login(acct, format!("Registered new user: {}\n", name)),
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
                Ok(acct) => return Login(acct, format!("Successfully logged in as {}\n", name)),
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
