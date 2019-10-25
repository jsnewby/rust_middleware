CREATE TABLE IF NOT EXISTS spend_transactions (
       id SERIAL,
       sender_id VARCHAR,
       recipient_id VARCHAR,
       transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE);

CREATE TABLE IF NOT EXISTS associated_accounts (
       id SERIAL,
       transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
       name_hash VARCHAR,
       aeternity_id VARCHAR);

CREATE INDEX IF NOT EXISTS associated_accounts_name_hash_idx ON associated_accounts(name_hash);
CREATE INDEX IF NOT EXISTS associated_accounts_account_id_idx ON associated_accounts(aeternity_id);
