CREATE VIEW full_nonprofits AS (
  SELECT nonprofit,
    affiliate
  FROM nonprofits AS nonprofit
    LEFT OUTER JOIN affiliates AS affiliate USING (affiliate_id)
);