-- Your SQL goes here
CREATE TABLE USERS(
    ID SERIAL PRIMARY KEY NOT NULL,
    USERNAME VARCHAR NOT NULL,
    PASS VARCHAR(64) NOT NULL
);