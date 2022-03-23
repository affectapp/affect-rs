SELECT *
FROM nonprofits
WHERE name ILIKE CONCAT('%', $1::text, '%')
ORDER BY create_time ASC,
  nonprofit_id ASC
LIMIT $2