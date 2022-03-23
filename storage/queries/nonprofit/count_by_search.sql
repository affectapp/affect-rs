SELECT COUNT(*) AS count
FROM nonprofits
WHERE name ILIKE CONCAT('%', $1::text, '%')