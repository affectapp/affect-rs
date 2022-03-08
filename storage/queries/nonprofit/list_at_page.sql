SELECT *
FROM nonprofits
WHERE (create_time, nonprofit_id) >= ($1, $2)
ORDER BY create_time ASC,
  nonprofit_id ASC
LIMIT $3