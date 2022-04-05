SELECT cause AS "cause!: _",
  cause_recipients AS "cause_recipients!: _"
FROM full_causes
WHERE (cause).user_id = $2
ORDER BY (cause).create_time ASC,
  (cause).cause_id ASC
LIMIT $1