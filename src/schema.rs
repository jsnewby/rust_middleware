table! {
    key_blocks (id) {
        id -> Int4,
        hash -> Nullable<Varchar>,
        height -> Nullable<Int8>,
        miner -> Nullable<Varchar>,
        beneficiary -> Nullable<Varchar>,
        pow -> Nullable<Varchar>,
        nonce -> Nullable<Numeric>,
        prev_hash -> Nullable<Varchar>,
        prev_key_hash -> Nullable<Varchar>,
        state_hash -> Nullable<Varchar>,
        target -> Nullable<Int8>,
        #[sql_name="time_"]
        time -> Nullable<Int8>,
        version -> Nullable<Int4>,
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
        state_hash -> Varchar,
        txs_hash -> Varchar,
        version -> Int4,
    }
}

table! {
    transactions (id) {
        id -> Int4,
        micro_block_id -> Nullable<Int4>,
        block_height -> Int4,
        block_hash -> Varchar,
        hash -> Varchar,
        signatures -> Text,
        tx_type -> Varchar,
        tx -> Jsonb,
        fee -> Int8,
        size -> Int4,
        valid -> Bool,
    }
}

joinable!(micro_blocks -> key_blocks (key_block_id));
joinable!(transactions -> micro_blocks (micro_block_id));

allow_tables_to_appear_in_same_query!(key_blocks, micro_blocks, transactions,);
