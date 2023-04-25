-- Your SQL goes here
CREATE TABLE IF NOT EXISTS servers (
	server_id BIGINT NOT NULL,
	PRIMARY KEY (server_id)
);
CREATE TABLE IF NOT EXISTS users (
    user_id BIGINT NOT NULL,
    PRIMARY KEY (user_id)
);
CREATE TABLE IF NOT EXISTS members (
    member_id SERIAL NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    server_id BIGINT NOT NULL REFERENCES servers (server_id) ON DELETE CASCADE,
    PRIMARY KEY (member_id)
);
CREATE TABLE IF NOT EXISTS channels (
    channel_id BIGINT NOT NULL,
    server_id BIGINT NOT NULL REFERENCES servers (server_id) ON DELETE CASCADE,
    PRIMARY KEY (channel_id)
);
CREATE TABLE IF NOT EXISTS tracked_channels (
    channel_id BIGINT NOT NULL REFERENCES channels (channel_id) ON DELETE CASCADE,
    PRIMARY KEY (channel_id)
);
CREATE TABLE IF NOT EXISTS tracked_members (
    member_id INTEGER NOT NULL REFERENCES members (member_id) ON DELETE CASCADE,
    PRIMARY KEY (member_id)
);
CREATE TABLE IF NOT EXISTS tokens (
    token_id SERIAL NOT NULL,
    token TEXT NOT NULL,
    PRIMARY KEY (token_id),
    UNIQUE (token)    
);
CREATE TABLE IF NOT EXISTS member_tokens (
    token_id INTEGER NOT NULL REFERENCES tokens (token_id) ON DELETE CASCADE,
    member_id INTEGER NOT NULL REFERENCES members (member_id) ON DELETE CASCADE,
    next_token_id INTEGER NOT NULL REFERENCES tokens (token_id) ON DELETE CASCADE,
    frequency BIGINT NOT NULL,
    PRIMARY KEY (member_id, token_id, next_token_id)
);