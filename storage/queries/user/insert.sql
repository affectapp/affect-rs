INSERT INTO users (
    user_id,
    create_time,
    update_time,
    firebase_uid,
    firebase_email,
    stripe_customer_id
  )
VALUES (DEFAULT, $1, $2, $3, $4, $5)
RETURNING *