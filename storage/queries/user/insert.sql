INSERT INTO users (
    create_time,
    update_time,
    firebase_uid,
    firebase_email
  )
VALUES ($1, $2, $3, $4)
RETURNING *