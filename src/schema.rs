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
    likes (user_id, blog_id) {
        user_id -> Varchar,
        blog_id -> Int4,
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

joinable!(blogs -> users (created_by));
joinable!(likes -> blogs (blog_id));
joinable!(likes -> users (user_id));

allow_tables_to_appear_in_same_query!(
    blogs,
    likes,
    users,
);
