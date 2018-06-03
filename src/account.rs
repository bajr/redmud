use diesel::prelude::*;

#[derive(Queryable)]
pub struct Account {
    name: String,
    salt: String,
    hash: String,
    //created: ,
    //lastseen: ,
}
