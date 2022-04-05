CREATE VIEW full_causes AS (
  SELECT cause,
    ARRAY_AGG(cause_recipients) AS cause_recipients
  FROM causes AS cause
    JOIN cause_recipients USING (cause_id)
  GROUP BY 1
);