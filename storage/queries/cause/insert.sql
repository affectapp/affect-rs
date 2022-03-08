INSERT INTO causes (
    cause_id,
    create_time,
    update_time,
    user_id,
    name
  )
VALUES (DEFAULT, $1, $2, $3, $4)
RETURNING *