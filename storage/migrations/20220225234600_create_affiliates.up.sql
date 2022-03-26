CREATE TYPE business_type AS ENUM ('individual', 'company', 'non_profit', 'government_entity');
CREATE TABLE affiliates (
  affiliate_id uuid NOT NULL DEFAULT uuid_generate_v4(),
  create_time TIMESTAMPTZ NOT NULL,
  update_time TIMESTAMPTZ NOT NULL,
  stripe_account_id VARCHAR(255) NOT NULL,
  company_name VARCHAR(255) NOT NULL,
  contact_email VARCHAR(255) NOT NULL,
  business_type business_type NOT NULL,
  PRIMARY KEY (affiliate_id)
)
