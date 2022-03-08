CREATE TABLE items (
  item_id uuid NOT NULL DEFAULT uuid_generate_v4(),
  create_time TIMESTAMPTZ NOT NULL,
  update_time TIMESTAMPTZ NOT NULL,
  user_id uuid NOT NULL,
  plaid_item_id VARCHAR(255) NOT NULL,
  plaid_access_token VARCHAR(255) NOT NULL,
  PRIMARY KEY (item_id),
  CONSTRAINT fk_item_to_user FOREIGN KEY (user_id) REFERENCES users(user_id)
)