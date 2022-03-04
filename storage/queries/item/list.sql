SELECT *
FROM items
WHERE user_id = $2
ORDER BY create_time ASC
LIMIT $1