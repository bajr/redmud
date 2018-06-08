use argon2rs::*;
use diesel::prelude::*;
use rand::{thread_rng, Rng};

use std::time::SystemTime;

use schema::accounts;

#[derive(Queryable, Insertable, Debug)]
#[table_name = "accounts"]
pub struct Account {
    name: String,
    email: Option<String>,
    valid: bool,
    salt: Vec<u8>,
    hash: Vec<u8>,
    created: SystemTime,
    lastseen: SystemTime,
}

impl Account {
    pub fn new(name: String, passwd: String) -> Account {
        // Generate a random, 32-byte salt
        let mut salt = vec![0u8; 32];
        thread_rng().fill(&mut salt[..]);
        // Generate a hash from the password and salt
        let mut hash = vec![0u8; 32];
        let argon = Argon2::default(Variant::Argon2i);
        argon.hash(&mut hash, passwd.as_bytes(), &salt, &[], &[]);

        Account {
            name,
            email: Some("".to_string()),
            valid: false,
            salt,
            hash,
            created: SystemTime::now(),
            lastseen: SystemTime::now(),
        }
    }
}
