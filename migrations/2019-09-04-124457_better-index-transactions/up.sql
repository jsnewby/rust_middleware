create index if not exists transactions_id_height_desc_idx on transactions (block_height desc, id desc);
