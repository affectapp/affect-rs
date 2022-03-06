SELECT *
FROM items
WHERE (create_time, item_id) >= ($1, $2)
  AND user_id = $3
ORDER BY create_time ASC,
  item_id ASC
LIMIT $4