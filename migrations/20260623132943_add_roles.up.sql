-- Add up migration script here
ALTER TABLE users
ADD role varchar NOT NULL DEFAULT 'user';;