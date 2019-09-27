table! {
    channel_identifiers (id) {
        id -> Int4,
        channel_identifier -> Nullable<Varchar>,
        transaction_id -> Int4,
    }
}

table! {
    contract_calls (id) {
        id -> Int4,
        transaction_id -> Int4,
        contract_id -> Varchar,
        caller_id -> Varchar,
        arguments -> Jsonb,
        callinfo -> Nullable<Jsonb>,
        result -> Nullable<Jsonb>,
    }
}

table! {
    contract_identifiers (id) {
        id -> Int4,
        contract_identifier -> Nullable<Varchar>,
        transaction_id -> Int4,
    }
}

table! {
    key_blocks (id) {
        id -> Int4,
        hash -> Nullable<Varchar>,
        height -> Nullable<Int8>,
        miner -> Nullable<Varchar>,
        beneficiary -> Nullable<Varchar>,
        nonce -> Nullable<Numeric>,
        pow -> Nullable<Text>,
        prev_hash -> Nullable<Varchar>,
        prev_key_hash -> Nullable<Varchar>,
        state_hash -> Nullable<Varchar>,
        target -> Nullable<Int8>,
        time_ -> Nullable<Int8>,
        version -> Nullable<Int4>,
        info -> Nullable<Varchar>,
    }
}

table! {
    micro_blocks (id) {
        id -> Int4,
        key_block_id -> Int4,
        hash -> Varchar,
        pof_hash -> Varchar,
        prev_hash -> Varchar,
        prev_key_hash -> Varchar,
        signature -> Varchar,
        time_ -> Nullable<Int8>,
        state_hash -> Varchar,
        txs_hash -> Varchar,
        version -> Int4,
    }
}

table! {
    names (id) {
        id -> Int4,
        name -> Varchar,
        name_hash -> Varchar,
        created_at_height -> Int8,
        owner -> Nullable<Varchar>,
        expires_at -> Int8,
        pointers -> Nullable<Jsonb>,
        tx_hash -> Varchar,
        transaction_id -> Int4,
    }
}

table! {
    oracle_queries (id) {
        id -> Int4,
        oracle_id -> Nullable<Varchar>,
        query_id -> Nullable<Varchar>,
        transaction_id -> Int4,
    }
}

table! {
    transactions (id) {
        id -> Int4,
        micro_block_id -> Nullable<Int4>,
        block_height -> Int4,
        block_hash -> Varchar,
        hash -> Varchar,
        signatures -> Nullable<Text>,
        tx_type -> Varchar,
        tx -> Jsonb,
        fee -> Numeric,
        size -> Int4,
        valid -> Bool,
        encoded_tx -> Nullable<Varchar>,
    }
}

joinable!(channel_identifiers -> transactions (transaction_id));
joinable!(contract_calls -> transactions (transaction_id));
joinable!(contract_identifiers -> transactions (transaction_id));
joinable!(micro_blocks -> key_blocks (key_block_id));
joinable!(names -> transactions (transaction_id));
joinable!(oracle_queries -> transactions (transaction_id));
joinable!(transactions -> micro_blocks (micro_block_id));

allow_tables_to_appear_in_same_query!(
    channel_identifiers,
    contract_calls,
    contract_identifiers,
    key_blocks,
    micro_blocks,
    names,
    oracle_queries,
    transactions,
);
