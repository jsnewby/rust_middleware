-- Your SQL goes here

CREATE TABLE contract_identifiers (
       id SERIAL PRIMARY KEY,
       contract_identifier VARCHAR(55),
       transaction_id INTEGER NOT NULL REFERENCES transactions(id));

CREATE INDEX contract_identifiers_contract_identifier ON contract_identifiers(contract_identifier);
CREATE INDEX contract_identifiers_transaction_id ON contract_identifiers(transaction_id);
