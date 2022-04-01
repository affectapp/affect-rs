CREATE VIEW full_affiliates AS (
  SELECT affiliate,
    asserted_nonprofit,
    ARRAY_AGG(affiliate_managers) AS affiliate_managers
  FROM affiliates AS affiliate
    JOIN affiliate_managers USING (affiliate_id)
    JOIN nonprofits AS asserted_nonprofit ON (
      asserted_nonprofit.nonprofit_id = affiliate.asserted_nonprofit_id
    )
  GROUP BY 1,
    2
);