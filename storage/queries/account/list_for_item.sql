SELECT *
FROM accounts
WHERE item_id = $1
ORDER BY create_time ASC