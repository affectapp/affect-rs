SELECT *
FROM users
WHERE (create_time, user_id) >= ($1, $2)
ORDER BY create_time ASC,
  user_id ASC
LIMIT $3