;;
CREATE TABLE IF NOT EXISTS hard_forks (
       id SERIAL PRIMARY KEY,
       name VARCHAR NOT NULL,
       height BIGINT UNIQUE
);

CREATE INDEX hard_forks_name_index ON hard_forks(name);
INSERT INTO hard_forks(name, height) values ('lima', 0);

DROP FUNCTION IF EXISTS get_fork_height;

CREATE FUNCTION
get_fork_height(name VARCHAR) RETURNS BIGINT
AS $$ SELECT height FROM hard_forks WHERE name = $1 $$ LANGUAGE SQL;

;; Below all_names has the very important MAX(block_height) which restricts
;; to the most recent auction. Joining on this in the 2 following views
;; also restricts them to blocks after the lima fork.
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
	block_height > get_fork_height('lima')
GROUP BY tx->>'name';

CREATE OR REPLACE VIEW winning_bids AS
SELECT
	t.tx->>'name' AS name,
	(MAX(t.tx->>'name_fee'))::numeric AS winning_bid
FROM
	transactions t JOIN all_names an ON t.tx->>'name' = an.name
WHERE
	t.block_height >= an.start_block_height AND
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
