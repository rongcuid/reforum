SELECT *
FROM
    topics t
    JOIN posts p ON p.topic_id = t.id
    JOIN