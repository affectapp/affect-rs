SELECT *
FROM items
WHERE (create_time) >= ($1)
  AND user_id = $3
ORDER BY create_time ASC
LIMIT $2