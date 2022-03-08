CREATE TABLE cause_recipients (
  cause_id uuid NOT NULL,
  nonprofit_id uuid NOT NULL,
  create_time TIMESTAMPTZ NOT NULL,
  update_time TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (cause_id, nonprofit_id)
)