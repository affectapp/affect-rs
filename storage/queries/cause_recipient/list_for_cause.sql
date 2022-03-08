SELECT *
FROM cause_recipients
WHERE cause_id = $1
ORDER BY create_time ASC,
  nonprofit_id ASC