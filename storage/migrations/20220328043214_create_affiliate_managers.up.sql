CREATE TABLE affiliate_managers (
  affiliate_id uuid NOT NULL,
  user_id uuid NOT NULL,
  create_time TIMESTAMPTZ NOT NULL,
  update_time TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (affiliate_id, user_id),
  CONSTRAINT fk_affiliate_manager_to_affiliate FOREIGN KEY (affiliate_id) REFERENCES affiliates(affiliate_id),
  CONSTRAINT fk_affiliate_manager_to_user FOREIGN KEY (user_id) REFERENCES users(user_id)
)