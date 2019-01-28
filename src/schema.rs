table! {
    members (id) {
        id -> Int4,
        user_id -> Int4,
        room_id -> Int4,
    }
}

table! {
    rooms (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        position -> Point,
    }
}

joinable!(members -> rooms (room_id));
joinable!(members -> users (user_id));

allow_tables_to_appear_in_same_query!(
    members,
    rooms,
    users,
);
