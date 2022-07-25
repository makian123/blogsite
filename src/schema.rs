table! {
    blogs (id) {
        id -> Int4,
        title -> Varchar,
        body -> Varchar,
        created_by -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        likes -> Int4,
    }
}

table! {
    users (id) {
        id -> Varchar,
        username -> Varchar,
        pass -> Varchar,
        is_admin -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    blogs,
    users,
);
