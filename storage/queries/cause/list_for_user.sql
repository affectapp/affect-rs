SELECT causes.*,
  COALESCE(ARRAY_AGG(cause_recipients), '{}') AS "recipients!: _"
FROM causes
  JOIN cause_recipients USING (cause_id)
WHERE user_id = $2
GROUP BY causes.cause_id
ORDER BY causes.create_time ASC,
  causes.cause_id ASC
LIMIT $1