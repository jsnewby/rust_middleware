

CREATE TABLE transactions (
       id SERIAL PRIMARY KEY,
       block_id INT REFERENCES blocks(id),
       original_json TEXT NOT NULL,
       recipient_pubkey VARCHAR(55),
       amount BIGINT NULL,
       fee BIGINT,
       ttl BIGINT,
       sender VARCHAR(55),
       payload TEXT NULL    

);
       
