use diesel::prelude::*;

use std::collections::BTreeMap;
use std::iter::Iterator;
use std::str::SplitWhitespace;

use account::*;
use shared::*;

type PlayFn = fn(&mut SplitWhitespace) -> PlayAction;

#[derive(Debug)]
pub enum PlayAction {
    Move,
    Quit,
    Noop(String),
}

pub use self::PlayAction::*;

/// Tables of recognized commands during play
lazy_static! {
    static ref DIRECTIONS: BTreeMap<&'static str, PlayFn> = {
        let mut m = BTreeMap::new();
        m.insert("north", go as PlayFn);
        m.insert("n", go as PlayFn);
        m.insert("northeast", go as PlayFn);
        m.insert("ne", go as PlayFn);
        m.insert("east", go as PlayFn);
        m.insert("e", go as PlayFn);
        m.insert("southeast", go as PlayFn);
        m.insert("se", go as PlayFn);
        m.insert("south", go as PlayFn);
        m.insert("s", go as PlayFn);
        m.insert("southwest", go as PlayFn);
        m.insert("sw", go as PlayFn);
        m.insert("west", go as PlayFn);
        m.insert("w", go as PlayFn);
        m.insert("northwest", go as PlayFn);
        m.insert("nw", go as PlayFn);
        m.insert("up", go as PlayFn);
        m.insert("u", go as PlayFn);
        m.insert("down", go as PlayFn);
        m.insert("d", go as PlayFn);
        m.insert("in", go as PlayFn);
        m.insert("out", go as PlayFn);
        m.insert("left", go as PlayFn);
        m.insert("l", go as PlayFn);
        m.insert("right", go as PlayFn);
        m.insert("r", go as PlayFn);
        m.insert("forward", go as PlayFn);
        m.insert("f", go as PlayFn);
        m.insert("backward", go as PlayFn);
        m.insert("back", go as PlayFn);
        m.insert("b", go as PlayFn);
        m
    };
    static ref PLAY_CMDS: BTreeMap<&'static str, PlayFn> = {
        let mut m = BTreeMap::new();
        m.insert("quit", quit as PlayFn);
        m.insert("logout", quit as PlayFn);
        //m.insert("stats", stats as CmdFn);
        //m.insert("who", who as CmdFn);
        m
    };
}

fn go(_line: &mut SplitWhitespace) -> PlayAction {
    unimplemented!();
}

fn quit(_line: &mut SplitWhitespace) -> PlayAction {
    unimplemented!();
}

pub fn cmd_playing(input: String) -> PlayAction {
    let mut line = input.split_whitespace();
    if let Some(cmd) = line.next() {
        let cmd_match: Vec<&str> = PLAY_CMDS
            .keys()
            .filter(|k| k.starts_with(cmd))
            .map(|&s| s)
            .collect();
        if cmd_match.is_empty() {
            Noop(format!("Unrecognized command: '{}'", cmd))
        } else if cmd_match.len() > 1 {
            Noop(format!(
                "Ambiguous command: {:?}\nMatches:{:?}\n",
                cmd, cmd_match
            ))
        } else {
            let func = PLAY_CMDS.get(cmd_match.first().unwrap()).unwrap();
            func(&mut line)
        }
    } else {
        Noop("".to_string())
    }
}
