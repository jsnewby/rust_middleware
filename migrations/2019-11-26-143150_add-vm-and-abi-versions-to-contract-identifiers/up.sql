ALTER TABLE contract_identifiers
ADD COLUMN abi_version INTEGER NOT NULL DEFAULT 1,
ADD COLUMN vm_version INTEGER NOT NULL DEFAULT 1;