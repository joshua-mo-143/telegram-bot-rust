-- Add migration script here
DROP TABLE IF EXISTS links;

CREATE TABLE IF NOT EXISTS links (
    id SERIAL PRIMARY KEY,
    url VARCHAR NOT NULL,
    status VARCHAR NOT NULL,
    user_id VARCHAR NOT NULL
);