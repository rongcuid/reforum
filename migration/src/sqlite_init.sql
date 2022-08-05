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