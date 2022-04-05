CREATE VIEW full_nonprofits AS (
  SELECT nonprofit,
    full_affiliate
  FROM nonprofits AS nonprofit
    LEFT OUTER JOIN full_affiliates AS full_affiliate ON (
      nonprofit.affiliate_id = (full_affiliate.affiliate).affiliate_id
    )
);