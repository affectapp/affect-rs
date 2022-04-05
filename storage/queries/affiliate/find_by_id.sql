SELECT affiliate AS "affiliate!: _",
  asserted_nonprofit AS "asserted_nonprofit: _",
  affiliate_managers AS "affiliate_managers!: _"
FROM full_affiliates
WHERE (affiliate).affiliate_id = $1