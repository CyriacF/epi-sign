// @generated automatically by Diesel CLI.

diesel::table! {
    cookies (id) {
        id -> Text,
        date -> Date,
        cookie_data -> Jsonb,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        username -> Text,
        password_hash -> Text,
        jwt_intra_epitech -> Nullable<Text>,
        jwt_expires_at -> Nullable<Timestamp>,
        signature_manuscrite -> Nullable<Text>,
    }
}

diesel::table! {
    edsquare_cookies (id) {
        id -> Text,
        user_id -> Text,
        date -> Date,
        cookie_data -> Jsonb,
    }
}

diesel::table! {
    edsquare_credentials (id) {
        id -> Text,
        user_id -> Text,
        email -> Text,
        password -> Text,
    }
}

diesel::table! {
    user_signatures (id) {
        id -> Text,
        user_id -> Text,
        signature_data -> Text,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cookies,
    users,
    edsquare_cookies,
    edsquare_credentials,
    user_signatures,
);
