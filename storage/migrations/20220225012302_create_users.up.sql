CREATE TABLE users (
    user_id uuid NOT NULL DEFAULT uuid_generate_v4(),
    create_time TIMESTAMPTZ NOT NULL,
    update_time TIMESTAMPTZ NOT NULL,
    firebase_uid VARCHAR(255) NOT NULL UNIQUE,
    firebase_email VARCHAR(255) NOT NULL,
    PRIMARY KEY (user_id)
)