-- Create tables
CREATE TABLE users(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    phc TEXT,
    post_signature TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    last_seen_at TIMESTAMP,
    last_post_at TIMESTAMP,
    muted_until TIMESTAMP,
    banned_at TIMESTAMP
);

CREATE TABLE moderators(
    moderator_user_id INTEGER PRIMARY KEY NOT NULL REFERENCES users(id) ON DELETE RESTRICT ON UPDATE CASCADE,
    assigned_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE past_moderators(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    moderator_user_id INTEGER NOT NULL REFERENCES users(id),
    unassigned_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reason TEXT NOT NULL
);

CREATE TABLE topics(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    author_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    title TEXT NOT NULL,
    number_posts INTEGER NOT NULL DEFAULT 0,
    public BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    deleted_at TIMESTAMP,
    last_updated_by INTEGER REFERENCES users(id) ON UPDATE CASCADE,
    views_from_users INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE posts(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE ON UPDATE CASCADE,
    author_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    body TEXT NOT NULL,
    post_number INTEGER,
    public BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    deleted_at TIMESTAMP,
    last_updated_by INTEGER REFERENCES users(id) ON UPDATE CASCADE
);

CREATE TABLE replies(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE ON UPDATE CASCADE,
    author_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    body TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_sessions(
    id BLOB PRIMARY KEY,
    session_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP
);

-- Insert triggers
CREATE TRIGGER tr_posts_after_insert BEFORE
INSERT
    ON posts BEGIN -- Set post number
UPDATE
    posts
SET
    post_number = (
        SELECT
            number_posts
        FROM
            topics
        WHERE
            topics.id = NEW.topic_id
    )
WHERE
    rowid = NEW.rowid;

UPDATE
    topics
SET
    number_posts = number_posts + 1
WHERE
    id = NEW.topic_id;

END;

-- Update triggers
CREATE TRIGGER tr_users_after_update
AFTER
UPDATE
    ON users BEGIN
UPDATE
    users
SET
    updated_at = CURRENT_TIMESTAMP
WHERE
    users.id = NEW.id;

END;

CREATE TRIGGER tr_topics_after_update
AFTER
UPDATE
    ON topics BEGIN
UPDATE
    topics
SET
    updated_at = CURRENT_TIMESTAMP
WHERE
    topics.id = NEW.id;

END;

CREATE TRIGGER tr_posts_after_update
AFTER
UPDATE
    ON posts BEGIN
UPDATE
    posts
SET
    updated_at = CURRENT_TIMESTAMP
WHERE
    posts.id = NEW.id;

END;

-- Delete triggers
-- Some seeds
INSERT INTO
    users(username, phc)
VALUES
    ('admin', '$argon2i$v=19$m=16,t=2,p=1$ZHdMaHdYeE1JZ3d6dmo0WQ$SWvpjaTUlShdvYL6qKARQg');