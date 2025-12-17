-- Add migration script here

CREATE TYPE watch_status AS ENUM ('active', 'fulfilled', 'cancelled');

CREATE TABLE watch_list(
id uuid NOT NULL, 
PRIMARY KEY(id), 
name VARCHAR NOT NULL,
user_id VARCHAR NOT NULL,
channel_id VARCHAR NOT NULL,
thread_id VARCHAR NOT NULL,
status watch_status NOT NULL, 
created_at timestamptz NOT NULL, 
updated_at timestamptz NOT NULL, 
expires_at timestamptz,
last_checked_at timestamptz
);

-- user can't watch the same name twice
CREATE UNIQUE INDEX unique_active_watch 
ON watch_list (user_id, name) 
WHERE status = 'active';
