SELECT nonprofit AS "nonprofit!: _",
  full_affiliate AS "full_affiliate: _"
FROM full_nonprofits
WHERE (nonprofit).nonprofit_id = $1