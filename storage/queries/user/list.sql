SELECT *
FROM users
ORDER BY create_time ASC,
  user_id ASC
LIMIT $1