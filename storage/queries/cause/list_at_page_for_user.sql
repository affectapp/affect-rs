SELECT causes.*,
  COALESCE(ARRAY_AGG(cause_recipients), '{}') AS "recipients!: _"
FROM causes
  JOIN cause_recipients USING (cause_id)
WHERE (causes.create_time, causes.cause_id) >= ($1, $2)
  AND causes.user_id = $3
GROUP BY causes.cause_id
ORDER BY causes.create_time ASC,
  causes.cause_id ASC
LIMIT $4