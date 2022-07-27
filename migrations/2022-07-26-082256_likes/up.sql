-- Your SQL goes here
CREATE TABLE likes(
    user_id VARCHAR REFERENCES users (id),
    blog_id INT REFERENCES blogs (id),
    CONSTRAINT user_blog_pkey PRIMARY KEY (user_id, blog_id)
);