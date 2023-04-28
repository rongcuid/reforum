-- (user_id)
SELECT
    banned_at
    muted_until,
    m.assigned_at moderator_assigned_at
FROM
    users u
    LEFT JOIN moderators m ON u.id = m.moderator_user_id
WHERE
    u.id = ?