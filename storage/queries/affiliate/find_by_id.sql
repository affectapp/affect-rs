SELECT affiliate_id,
  affiliates.create_time,
  affiliates.update_time,
  stripe_account_id,
  company_name,
  contact_email,
  business_type as "business_type: _",
  COALESCE(ARRAY_AGG(affiliate_managers), '{}') AS "affiliate_managers!: _"
FROM affiliates
  JOIN affiliate_managers USING (affiliate_id)
WHERE affiliate_id = $1
GROUP BY affiliate_id