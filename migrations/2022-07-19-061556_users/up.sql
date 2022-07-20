-- Your SQL goes here
CREATE TABLE USERS(
    id varchar(36) PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4 (),
    username VARCHAR NOT NULL,
    pass VARCHAR(64) NOT NULL,
    is_admin BOOLEAN DEFAULT FALSE NOT NULL
);