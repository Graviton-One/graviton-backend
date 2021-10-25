table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    chains (id) {
        id -> Int8,
        name -> Varchar,
        coingeco_id -> Varchar,
        code -> Varchar,
        chain_type -> Chain_type_enum,
        rpc_url -> Varchar,
        explorer_url -> Varchar,
        gton_address -> Varchar,
        meta -> Json,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    dexes (id) {
        id -> Int8,
        chain_id -> Int8,
        router_info -> Json,
        name -> Varchar,
        meta -> Json,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    farms (id) {
        id -> Int8,
        pool_id -> Int8,
        tvl -> Float8,
        total_locked -> Numeric,
        alloc_point -> Numeric,
        apy -> Float8,
        total_rewards -> Float8,
        meta -> Json,
        farm_index -> Int8,
        last_updated -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    gton_price (id) {
        id -> Int8,
        price_hour -> Timestamp,
        price -> Float8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    pollers (id) {
        id -> Int8,
        block_num -> Int8,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    pools (id) {
        id -> Int8,
        dex_id -> Int8,
        pool_address -> Varchar,
        token_a_address -> Varchar,
        token_b_address -> Varchar,
        tvl -> Float8,
        reserve_a -> Numeric,
        reserve_b -> Numeric,
        fee -> Nullable<Numeric>,
        lp_total_supply -> Numeric,
        lp_fee -> Nullable<Numeric>,
        meta -> Json,
        last_updated -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db_types::*;

    staking (id) {
        id -> Int8,
        chain_id -> Int8,
        tvl -> Float8,
        total_locked -> Numeric,
        alloc_point -> Numeric,
        apy -> Float8,
        total_rewards -> Float8,
        meta -> Json,
        farm_index -> Int8,
        last_updated -> Timestamp,
    }
}

joinable!(dexes -> chains (chain_id));
joinable!(farms -> pools (pool_id));
joinable!(pools -> dexes (dex_id));
joinable!(staking -> chains (chain_id));

allow_tables_to_appear_in_same_query!(
    chains,
    dexes,
    farms,
    gton_price,
    pollers,
    pools,
    staking,
);
