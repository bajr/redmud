CREATE ROLE redmud LOGIN PASSWORD 'redmud';

GRANT CONNECT ON DATABASE redmuddb TO redmud;

CREATE TABLE accounts (
  name TEXT NOT NULL PRIMARY KEY,
  email TEXT,
  valid BOOLEAN NOT NULL DEFAULT false,
  salt BYTEA NOT NULL CHECK (octet_length(salt) > 8),
  hash BYTEA NOT NULL CHECK (octet_length(hash) = 32),
  created TIMESTAMP NOT NULL DEFAULT NOW(),
  lastseen TIMESTAMP NOT NULL
);

GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO redmud;

