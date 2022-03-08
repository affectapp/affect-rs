CREATE TABLE ach_credentials (
  account_id uuid NOT NULL,
  create_time TIMESTAMPTZ NOT NULL,
  update_time TIMESTAMPTZ NOT NULL,
  account_number VARCHAR(255) NOT NULL,
  routing_number VARCHAR(255) NOT NULL,
  wire_routing_number VARCHAR(255),
  PRIMARY KEY (account_id),
  CONSTRAINT fk_ach_credentials_to_account FOREIGN KEY (account_id) REFERENCES accounts(account_id)
)