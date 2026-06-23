-- Add down migration script here
-- sqlx migrate add -r <migration_name> --> for migration up and down create
DROP TABLE notes;
DROP TABLE users;