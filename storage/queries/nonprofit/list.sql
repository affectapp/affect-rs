SELECT *
FROM nonprofits
ORDER BY create_time ASC,
  nonprofit_id ASC
LIMIT $1