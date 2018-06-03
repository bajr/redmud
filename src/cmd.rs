use shared::*;
use std::collections::HashMap;
use std::iter::Iterator;
use std::str::SplitWhitespace;

type CmdFn = fn(&mut SplitWhitespace) -> Option<String>;

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

fn help(_line: &mut SplitWhitespace) -> Option<String> {
    Some(SPLASH.to_string())
}

fn quit(_line: &mut SplitWhitespace) -> Option<String> {
    None
}

fn register(line: &mut SplitWhitespace) -> Option<String> {
    if let Some(name) = line.next() {
        Some(format!("I found a name: {}\n", name))
    } else {
        Some(format!("No name given\n"))
    }
}

// Parse commands for players in Connected state
pub fn cmd_connected(input: String) -> Option<String> {
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
            func(&mut line)
        }
    } else {
        info!("We didn't get anything: {}", input);
        Some("".to_string())
    }
}
