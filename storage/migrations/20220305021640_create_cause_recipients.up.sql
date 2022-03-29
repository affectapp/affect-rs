CREATE TABLE cause_recipients (
  cause_id uuid NOT NULL,
  nonprofit_id uuid NOT NULL,
  create_time TIMESTAMPTZ NOT NULL,
  update_time TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (cause_id, nonprofit_id),
  CONSTRAINT fk_cause_recipient_to_cause FOREIGN KEY (cause_id) REFERENCES causes(cause_id),
  CONSTRAINT fk_cause_recipient_to_nonprofit FOREIGN KEY (nonprofit_id) REFERENCES nonprofits(nonprofit_id)
)