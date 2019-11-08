CREATE INDEX IF NOT EXISTS transactions_tx_name_idx ON transactions((tx->>'name'));
