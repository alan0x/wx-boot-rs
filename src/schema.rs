// @generated automatically by Diesel CLI.

diesel::table! {
    access_tokens (id) {
        id -> Int8,
        user_id -> Int8,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        #[max_length = 255]
        kind -> Varchar,
        #[max_length = 255]
        value -> Varchar,
        #[max_length = 255]
        device -> Nullable<Varchar>,
        expired_at -> Timestamptz,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    emails (id) {
        id -> Int8,
        user_id -> Int8,
        #[max_length = 255]
        value -> Varchar,
        #[max_length = 255]
        domain -> Varchar,
        is_verified -> Bool,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    help_tickets (id) {
        id -> Int8,
        owner_id -> Int8,
        #[max_length = 50]
        kind -> Varchar,
        #[max_length = 200]
        subject -> Varchar,
        label_ids -> Array<Nullable<Int8>>,
        is_recalled -> Bool,
        content -> Text,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
        is_resolved -> Bool,
        extra -> Nullable<Jsonb>,
        is_processed -> Bool,
    }
}

diesel::table! {
    notifications (id) {
        id -> Int8,
        owner_id -> Int8,
        sender_id -> Nullable<Int8>,
        #[max_length = 255]
        subject -> Varchar,
        body -> Varchar,
        #[max_length = 50]
        kind -> Varchar,
        is_read -> Bool,
        extra -> Jsonb,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    orders (id) {
        id -> Int8,
        #[max_length = 255]
        order_id -> Varchar,
        #[max_length = 255]
        paid_reason -> Varchar,
        amount -> Numeric,
        #[max_length = 50]
        trade_state -> Varchar,
        #[max_length = 255]
        payment_id -> Varchar,
        paid_at -> Nullable<Timestamptz>,
        paid_by -> Nullable<Int8>,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
        discount -> Jsonb,
        actual_amount -> Numeric,
    }
}

diesel::table! {
    security_codes (id) {
        id -> Int8,
        user_id -> Int8,
        #[max_length = 255]
        email -> Nullable<Varchar>,
        #[max_length = 255]
        value -> Varchar,
        #[max_length = 255]
        send_method -> Varchar,
        consumed_at -> Nullable<Timestamptz>,
        expired_at -> Timestamptz,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_last_login (id) {
        id -> Int8,
        user_id -> Int8,
        last_login -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Int8,
        #[max_length = 255]
        ident_name -> Varchar,
        #[max_length = 255]
        display_name -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        is_disabled -> Bool,
        disabled_by -> Nullable<Int8>,
        disabled_at -> Nullable<Timestamptz>,
        is_verified -> Bool,
        verified_at -> Nullable<Timestamptz>,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
        in_kernel -> Bool,
        #[max_length = 255]
        weixin_openid -> Nullable<Varchar>,
        profile -> Jsonb,
        #[max_length = 255]
        avatar -> Nullable<Varchar>,
        contribute -> Nullable<Int8>,
        enable_ranking -> Nullable<Bool>,
        latest_export -> Nullable<Timestamptz>,
        is_member -> Nullable<Bool>,
        expired_at -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    access_tokens,
    emails,
    help_tickets,
    notifications,
    orders,
    security_codes,
    user_last_login,
    users,
);
