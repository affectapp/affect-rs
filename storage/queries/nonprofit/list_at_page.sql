SELECT nonprofit AS "nonprofit!: _",
  affiliate AS "affiliate!: _"
FROM full_nonprofits
WHERE (
    (nonprofit).create_time,
    (nonprofit).nonprofit_id
  ) >= ($1, $2)
ORDER BY (nonprofit).create_time ASC,
  (nonprofit).nonprofit_id ASC
LIMIT $3