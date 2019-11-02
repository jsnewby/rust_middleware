
CREATE OR REPLACE VIEW winning_bids AS
SELECT
	t.tx->>'name' AS name,
	MAX((t.tx->>'name_fee')::numeric) AS winning_bid
FROM
	transactions t JOIN all_names an ON t.tx->>'name' = an.name
WHERE
	t.block_height >= an.start_block_height
GROUP BY
	tx->>'name';
