// @generated automatically by Diesel CLI.

diesel::table! {
    channels (channel_id) {
        channel_id -> Int8,
        server_id -> Int8,
    }
}

diesel::table! {
    member_tokens (member_id, token_id, next_token_id) {
        token_id -> Int4,
        member_id -> Int4,
        next_token_id -> Int4,
        frequency -> Int8,
    }
}

diesel::table! {
    members (member_id) {
        member_id -> Int4,
        user_id -> Int8,
        server_id -> Int8,
    }
}

diesel::table! {
    servers (server_id) {
        server_id -> Int8,
    }
}

diesel::table! {
    tokens (token_id) {
        token_id -> Int4,
        token -> Text,
    }
}

diesel::table! {
    tracked_channels (channel_id) {
        channel_id -> Int8,
    }
}

diesel::table! {
    tracked_members (member_id) {
        member_id -> Int4,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Int8,
    }
}

diesel::joinable!(channels -> servers (server_id));
diesel::joinable!(member_tokens -> members (member_id));
diesel::joinable!(members -> servers (server_id));
diesel::joinable!(members -> users (user_id));
diesel::joinable!(tracked_channels -> channels (channel_id));
diesel::joinable!(tracked_members -> members (member_id));

diesel::allow_tables_to_appear_in_same_query!(
    channels,
    member_tokens,
    members,
    servers,
    tokens,
    tracked_channels,
    tracked_members,
    users,
);
