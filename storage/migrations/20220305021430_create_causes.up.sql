CREATE TABLE causes (
  cause_id uuid NOT NULL DEFAULT uuid_generate_v4(),
  create_time TIMESTAMPTZ NOT NULL,
  update_time TIMESTAMPTZ NOT NULL,
  user_id uuid NOT NULL,
  name VARCHAR(255) NOT NULL,
  PRIMARY KEY (cause_id),
  CONSTRAINT fk_cause_to_user FOREIGN KEY (user_id) REFERENCES users(user_id)
)