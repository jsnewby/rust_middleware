table! {
    blocks (id) {
        id -> Int4,
        hash -> Nullable<Varchar>,
        height -> Nullable<Int8>,
        miner -> Nullable<Varchar>,
        nonce -> Nullable<Int8>,
        prev_hash -> Nullable<Varchar>,
        state_hash -> Nullable<Varchar>,
        target -> Nullable<Int8>,
        time_ -> Nullable<Int8>,
        txs_hash -> Nullable<Varchar>,
        version -> Nullable<Int4>,
    }
}

table! {
    transactions (id) {
        id -> Int4,
        block_id -> Nullable<Int4>,
        original_json -> Text,
        recipient_pubkey -> Nullable<Varchar>,
        amount -> Nullable<Int8>,
        fee -> Nullable<Int8>,
        ttl -> Nullable<Int8>,
        sender -> Nullable<Varchar>,
        payload -> Nullable<Text>,
    }
}

joinable!(transactions -> blocks (block_id));

allow_tables_to_appear_in_same_query!(
    blocks,
    transactions,
);
