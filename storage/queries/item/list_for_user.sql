SELECT *
FROM items
WHERE user_id = $2
ORDER BY create_time ASC,
  item_id ASC
LIMIT $1