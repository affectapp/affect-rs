SELECT *
FROM nonprofits
WHERE name LIKE CONCAT('%', $1::text, '%')
  AND (create_time, nonprofit_id) >= ($2, $3)
ORDER BY create_time ASC,
  nonprofit_id ASC
LIMIT $4