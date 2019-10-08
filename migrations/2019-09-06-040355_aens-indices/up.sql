create index if not exists transactions_tx_name_salt_idx on transactions((tx->>'name_salt'));
