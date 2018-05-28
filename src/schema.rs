table! {
    accounts (name) {
        name -> Text,
        salt -> Bytea,
        hash -> Bytea,
        created -> Timestamp,
        lastseen -> Timestamp,
    }
}
