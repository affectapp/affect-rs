SELECT affiliate_id,
  create_time,
  update_time,
  stripe_account_id,
  company_name,
  contact_email,
  business_type as "business_type: _"
FROM affiliates
WHERE affiliate_id = $1