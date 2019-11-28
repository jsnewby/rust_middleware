CREATE OR REPLACE FUNCTION maybeUpdateNameAuctionEntries() RETURNS VOID AS
$BODY$
DECLARE name_is_populated BOOLEAN;
BEGIN
SELECT relispopulated INTO name_is_populated FROM pg_class WHERE relname = 'name_auction_entries';
IF NOT name_is_populated THEN
REFRESH MATERIALIZED VIEW name_auction_entries;
END IF;
END
$BODY$
LANGUAGE 'plpgsql' ;
