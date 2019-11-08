DROP VIEW IF EXISTS name_auction_entries;

CREATE OR REPLACE VIEW name_auction_entries AS
SELECT
	an.name AS name,
	an.auction_expiration AS expiration,
	wb.winning_bid AS winning_bid,
	(t.tx->>'account_id')::VARCHAR AS winning_bidder,
	t.id AS transaction_id
FROM
	transactions t, all_names an, winning_bids wb
WHERE
	t.tx->>'name' = an.name AND
	t.tx->>'name' = wb.name AND
	(t.tx->>'name_fee')::numeric = wb.winning_bid;
