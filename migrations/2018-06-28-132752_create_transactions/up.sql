

CREATE TABLE transactions (
       id SERIAL PRIMARY KEY,
       block_id INT NOT NULL REFERENCES blocks(id),
       block_height INT NOT NULL,
       block_hash VARCHAR(55) NOT NULL,
       hash VARCHAR(55) NOT NULL,
       signatures TEXT NOT NULL,
       tx_type VARCHAR(64) NOT NULL,
       tx JSONB NOT NULL,
       valid BOOLEAN NOT NULL DEFAULT TRUE
);
       
CREATE INDEX transactions_tx_type_index ON transactions(tx_type);
