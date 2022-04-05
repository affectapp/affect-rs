SELECT nonprofit AS "nonprofit!: _",
  affiliate AS "affiliate!: _"
FROM full_nonprofits
WHERE (nonprofit).nonprofit_id = $1