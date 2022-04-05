SELECT COUNT(*) AS count
FROM full_causes
WHERE (cause).user_id = $1