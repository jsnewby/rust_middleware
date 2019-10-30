
CREATE OR REPLACE FUNCTION
lima_name_auction_timeout(name VARCHAR) RETURNS INT
AS $$
DECLARE timeout NUMERIC;
DECLARE _length NUMERIC = char_length(name) - 6;
DECLARE multiplier_day INT = 480;
BEGIN
        CASE
        WHEN _length >= 13 THEN timeout = 0;
        WHEN _length > 8 THEN timeout = multiplier_day;
        WHEN _length > 4 THEN
             timeout = multiplier_day * 31;
        ELSE timeout = multiplier_day * 62;
        END CASE;

        RETURN timeout;

END; $$
LANGUAGE PLPGSQL;
