table! {
    accounts (name) {
        name -> Text,
        email -> Nullable<Text>,
        valid -> Bool,
        salt -> Bytea,
        hash -> Bytea,
        created -> Timestamp,
        lastseen -> Timestamp,
    }
}
