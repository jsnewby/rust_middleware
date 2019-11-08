
CREATE OR REPLACE VIEW all_names AS
SELECT
	LOWER(tx->>'name') AS name,
	MAX(block_height) AS start_block_height,
	(MAX(block_height) + lima_name_auction_timeout(tx->>'name'))::BIGINT
			  AS auction_expiration
FROM transactions
WHERE
	tx_type='NameClaimTx' AND
	(tx->>'name_salt')::NUMERIC <> 0 AND
	block_height >= get_fork_height(4)
GROUP BY tx->>'name';

CREATE OR REPLACE VIEW max_bids

CREATE OR REPLACE VIEW winning_bids AS
SELECT
       an.name,
       MAX((t.tx->>'name_fee')::numeric) AS winning_bid,
       MAX(t.block_height) AS height
FROM
       transactions t JOIN all_names an ON LOWER(t.tx->>'name') = an.name
WHERE
        t.block_height >= an.start_block_height
GROUP BY
	an.name;

DROP VIEW IF EXISTS name_auction_entries;

CREATE MATERIALIZED VIEW name_auction_entries AS
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
 	LOWER(t.tx->>'name') = an.name AND
	LOWER(t.tx->>'name') = wb.name AND
        t.block_height >= an.start_block_height AND
	(t.tx->>'name_fee')::numeric = wb.winning_bid
WITH DATA;

COMMIT;

CREATE UNIQUE INDEX name_auction_entries_name_idx ON name_auction_entries(name);
