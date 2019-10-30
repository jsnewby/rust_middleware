ALTER TABLE names ADD COLUMN auction_end_height BIGINT;
UPDATE names SET auction_end_height = lima_name_auction_timeout(name) + created_at_height;
UPDATE names SET expires_at = lima_name_auction_timeout(name) + expires_at;
