CREATE TABLE key_blocks (
       id SERIAL PRIMARY KEY,
       hash VARCHAR(55),
       height BIGINT,
       miner VARCHAR(55),
       beneficiary VARCHAR(55),
       nonce numeric(20,0),
       pow TEXT,
       prev_hash VARCHAR(55),
       prev_key_hash VARCHAR(55),
       state_hash VARCHAR(55),
       target BIGINT,
       time_ BIGINT,
       version INTEGER);
