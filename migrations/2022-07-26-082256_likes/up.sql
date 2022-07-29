-- Your SQL goes here
CREATE TABLE likes(
    user_id VARCHAR REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    blog_id INT REFERENCES blogs(id) ON DELETE CASCADE NOT NULL,
    CONSTRAINT user_blog_pkey PRIMARY KEY (user_id, blog_id)
);