SELECT nonprofit AS "nonprofit!: _",
  affiliate AS "affiliate: _"
FROM full_nonprofits
WHERE (nonprofit).name ILIKE CONCAT('%', $1::text, '%')
ORDER BY (nonprofit).create_time ASC,
  (nonprofit).nonprofit_id ASC
LIMIT $2