CREATE TABLE IF NOT EXISTS spend_transactions (
       id SERIAL,
       sender_id VARCHAR,
       recipient_id VARCHAR,
       transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE);
