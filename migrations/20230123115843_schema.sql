-- Add migration script here
DROP TABLE IF EXISTS links;

CREATE TABLE IF NOT EXISTS links (
    id SERIAL PRIMARY KEY,
    url VARCHAR PRIMARY KEY,
    status VARCHAR PRIMARY KEY,
    user_id VARCHAR PRIMARY KEY
);