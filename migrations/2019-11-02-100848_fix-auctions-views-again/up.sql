
CREATE OR REPLACE VIEW winning_bids AS
SELECT
	t.tx->>'name' AS name,
	(MAX(t.tx->>'name_fee'))::numeric AS winning_bid,
	MAX(t.block_height) AS height
FROM
	transactions t JOIN all_names an ON t.tx->>'name' = an.name
WHERE
	t.block_height >= an.start_block_height
GROUP BY
	tx->>'name';

CREATE OR REPLACE VIEW name_auction_entries AS
SELECT
	an.name AS name,
	wb.height + lima_name_auction_timeout(an.name)::BIGINT
			  AS expiration,
	wb.winning_bid AS winning_bid,
	(t.tx->>'account_id')::VARCHAR AS winning_bidder,
	t.id AS transaction_id
FROM
	transactions t, all_names an, winning_bids wb
WHERE
	t.tx->>'name' = an.name AND
	t.tx->>'name' = wb.name AND
	(t.tx->>'name_fee')::numeric = wb.winning_bid;
