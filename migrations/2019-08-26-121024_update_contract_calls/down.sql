-- This file should undo anything in `up.sql`
ALTER TABLE contract_calls ADD column result JSONB;