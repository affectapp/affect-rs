SELECT *
FROM causes
WHERE user_id = $2
ORDER BY create_time ASC,
  cause_id ASC
LIMIT $1