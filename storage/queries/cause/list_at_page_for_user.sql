SELECT *
FROM causes
WHERE (create_time, cause_id) >= ($1, $2)
  AND user_id = $3
ORDER BY create_time ASC,
  cause_id ASC
LIMIT $4