-- Your SQL goes here

CREATE TABLE channel_identifiers (
       id SERIAL PRIMARY KEY,
       channel_identifier VARCHAR(55),
       transaction_id INTEGER NOT NULL REFERENCES transactions(id));

CREATE INDEX channel_identifiers_channel_identifier ON channel_identifiers(channel_identifier);
CREATE INDEX channel_identifiers_transaction_id ON channel_identifiers(transaction_id);
-- Your SQL goes here
