table! {
    blocks (id) {
        id -> Int4,
        hash -> Nullable<Varchar>,
        height -> Nullable<Int8>,
        miner -> Nullable<Varchar>,
        nonce -> Nullable<Numeric>,
        prev_hash -> Nullable<Varchar>,
        state_hash -> Nullable<Varchar>,
        target -> Nullable<Int8>,
        #[sql_name="time_"]
        time -> Nullable<Int8>,
        txs_hash -> Nullable<Varchar>,
        version -> Nullable<Int4>,
    }
}

table! {
    transactions (id) {
        id -> Int4,
        block_id -> Int4,
        block_height -> Int4,
        block_hash -> Varchar,
        hash -> Varchar,
        signatures -> Text,
        tx_type -> Text,
        tx -> Jsonb,
    }
}

joinable!(transactions -> blocks (block_id));

allow_tables_to_appear_in_same_query!(
    blocks,
    transactions,
);
