SELECT *
FROM nonprofits
WHERE name LIKE CONCAT('%', $1::text, '%')
ORDER BY create_time ASC,
  nonprofit_id ASC
LIMIT $2