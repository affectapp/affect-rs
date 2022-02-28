SELECT *
FROM nonprofits
WHERE (create_time) >= ($1)
ORDER BY create_time ASC
LIMIT $2