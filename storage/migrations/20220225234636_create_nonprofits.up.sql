CREATE TABLE nonprofits (
  nonprofit_id uuid NOT NULL DEFAULT uuid_generate_v4(),
  create_time TIMESTAMPTZ NOT NULL,
  update_time TIMESTAMPTZ NOT NULL,
  change_nonprofit_id VARCHAR(255) NOT NULL UNIQUE,
  icon_url VARCHAR(255) NOT NULL,
  title VARCHAR(255) NOT NULL,
  ein VARCHAR(255) NOT NULL,
  mission TEXT NOT NULL,
  category VARCHAR(255) NOT NULL,
  PRIMARY KEY (nonprofit_id)
);