SELECT cause AS "cause!: _",
  cause_recipients AS "cause_recipients!: _"
FROM full_causes
WHERE ((cause).create_time, (cause).cause_id) >= ($1, $2)
  AND (cause).user_id = $3
ORDER BY (cause).create_time ASC,
  (cause).cause_id ASC
LIMIT $4