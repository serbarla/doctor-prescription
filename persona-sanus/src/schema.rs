diesel::table! {
    users (id) {
        id -> Text,
        email -> Text,
        password_hash -> Text,
        display_name -> Text,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    sessions (id) {
        id -> Text,
        token_hash -> Text,
        user_id -> Text,
        expires_at -> Text,
        created_at -> Text,
    }
}

diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(users, sessions);
