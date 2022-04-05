SELECT nonprofit AS "nonprofit!: _",
  affiliate AS "affiliate: _"
FROM full_nonprofits
WHERE (nonprofit).name ILIKE CONCAT('%', $1::text, '%')
  AND (
    (nonprofit).create_time,
    (nonprofit).nonprofit_id
  ) >= ($2, $3)
ORDER BY (nonprofit).create_time ASC,
  (nonprofit).nonprofit_id ASC
LIMIT $4