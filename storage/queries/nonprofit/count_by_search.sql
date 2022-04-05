SELECT COUNT(*) AS "count!"
FROM full_nonprofits
WHERE (nonprofit).name ILIKE CONCAT('%', $1::text, '%')