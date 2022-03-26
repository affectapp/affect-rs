CREATE TABLE accounts (
  account_id uuid NOT NULL DEFAULT uuid_generate_v4(),
  create_time TIMESTAMPTZ NOT NULL,
  update_time TIMESTAMPTZ NOT NULL,
  item_id uuid NOT NULL,
  plaid_account_id VARCHAR(255) NOT NULL UNIQUE,
  name VARCHAR(255) NOT NULL,
  mask VARCHAR(255),
  stripe_bank_account_id VARCHAR(255) NOT NULL UNIQUE,
  PRIMARY KEY (account_id),
  CONSTRAINT fk_account_to_item FOREIGN KEY (item_id) REFERENCES items(item_id)
)