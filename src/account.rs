use argon2rs::*;
use diesel::insert_into;
use diesel::prelude::*;
use rand::{thread_rng, Rng};

use std::time::SystemTime;

use schema::accounts;
use shared::*;

#[derive(Queryable, Insertable, Debug)]
#[table_name = "accounts"]
pub struct Account {
    pub name: String,
    email: Option<String>,
    valid: bool,
    salt: Vec<u8>,
    hash: Vec<u8>,
    created: SystemTime,
    lastseen: SystemTime,
}

impl Account {
    /// Validate and register a new account and insert it into the database
    pub fn new(name: String, passwd: String) -> Result<Account, String> {
        use schema::accounts;

        let db_conn = SHARE.db_conn.get().unwrap();

        // Check if the account already exists
        if let Ok(_) = accounts::table.find(&name).first::<Account>(&*db_conn) {
            return Err(format!(
                "'{}' already exists. Please choose a different name.\n",
                name
            ));
        } else {
            // Generate a random, 32-byte salt
            let mut salt = vec![0u8; 32];
            thread_rng().fill(&mut salt[..]);

            // Generate a hash from the password and salt
            let mut hash = vec![0u8; 32];
            let argon = Argon2::default(Variant::Argon2i);
            argon.hash(&mut hash, passwd.as_bytes(), &salt, &[], &[]);

            let acct = Account {
                name,
                email: Some("".to_string()),
                valid: false,
                salt,
                hash,
                created: SystemTime::now(),
                lastseen: SystemTime::now(),
            };

            // Insert the account into the database
            if let Ok(_) = insert_into(accounts::table)
                .values(&acct)
                .execute(&*db_conn)
            {
                info!("Registered new user: {}", acct.name);
                return Ok(acct);
            } else {
                error!("Database error creating {:?}!", acct);
                return Err(format!("Database error creating user!\n"));
            }
        }
    }

    pub fn login(name: String, passwd: String) -> Result<Account, String> {
        use schema::accounts;

        let db_conn = SHARE.db_conn.get().unwrap();

        // Check if the account already exists
        if let Ok(acct) = accounts::table.find(&name).first::<Account>(&*db_conn) {
            let mut given_hash = vec![0u8; 32];
            let argon = Argon2::default(Variant::Argon2i);
            argon.hash(&mut given_hash, passwd.as_bytes(), &acct.salt, &[], &[]);

            if given_hash == acct.hash {
                info!("Successful login for: {}", acct.name);
                return Ok(acct);
            }
        }
        Err(format!("Invalid login.\n"))
    }
}
