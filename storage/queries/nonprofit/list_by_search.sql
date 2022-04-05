SELECT nonprofit AS "nonprofit!: _",
  full_affiliate AS "full_affiliate: _"
FROM full_nonprofits
WHERE (nonprofit).name ILIKE CONCAT('%', $1::text, '%')
ORDER BY (nonprofit).create_time ASC,
  (nonprofit).nonprofit_id ASC
LIMIT $2