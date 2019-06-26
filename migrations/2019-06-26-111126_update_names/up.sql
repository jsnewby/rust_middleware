-- Your SQL goes here
alter table names add column tx_hash VARCHAR(255) NOT NULL;
CREATE INDEX names_tx_hash_index ON names(tx_hash);