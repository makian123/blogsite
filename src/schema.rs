table! {
    blogs (id) {
        id -> Int4,
        title -> Varchar,
        body -> Varchar,
        creator_id -> Int4,
        created_time -> Int8,
        last_edited_time -> Int8,
        likes -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        pass -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    blogs,
    users,
);
