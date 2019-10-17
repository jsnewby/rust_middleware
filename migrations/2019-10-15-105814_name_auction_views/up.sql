DROP TABLE IF EXISTS protocols;

CREATE TABLE IF NOT EXISTS protocols (
       id SERIAL PRIMARY KEY,
       version BIGINT NOT NULL,
       effective_at_height BIGINT NOT NULL
);

CREATE INDEX protocols_version_index ON protocols(version);


DROP FUNCTION IF EXISTS get_fork_height;

CREATE OR REPLACE FUNCTION
get_fork_height(version INTEGER) RETURNS BIGINT
AS $$ SELECT effective_at_height FROM protocols WHERE version = $1 $$ LANGUAGE SQL;

CREATE OR REPLACE VIEW all_names AS
SELECT
	tx->>'name' AS name,
	MAX(block_height) AS start_block_height,
	(MAX(block_height) + lima_name_auction_timeout(tx->>'name'))::BIGINT
			  AS auction_expiration
FROM transactions
WHERE
	tx_type='NameClaimTx' AND
	(tx->>'name_salt')::NUMERIC = 0 AND
	block_height >= get_fork_height(4)
GROUP BY tx->>'name';

CREATE OR REPLACE VIEW winning_bids AS
SELECT
	t.tx->>'name' AS name,
	(MAX(t.tx->>'name_fee'))::numeric AS winning_bid
FROM
	transactions t JOIN all_names an ON t.tx->>'name' = an.name
WHERE
	t.block_height >= an.start_block_height
GROUP BY
	tx->>'name';

CREATE OR REPLACE VIEW name_auction_entries AS
SELECT
	an.name AS name,
	an.auction_expiration AS expiration,
	wb.winning_bid AS winning_bid,
	(t.tx->>'account_id')::VARCHAR AS winning_bidder
FROM
	transactions t, all_names an, winning_bids wb
WHERE
	t.tx->>'name' = an.name AND
	t.tx->>'name' = wb.name AND
	(t.tx->>'name_fee')::numeric = wb.winning_bid;
