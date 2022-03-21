SELECT COUNT(*) AS count
FROM nonprofits
WHERE name LIKE CONCAT('%', $1::text, '%')