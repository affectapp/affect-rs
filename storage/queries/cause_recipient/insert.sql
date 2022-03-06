INSERT INTO cause_recipients (
    cause_id,
    nonprofit_id,
    create_time,
    update_time
  )
VALUES ($1, $2, $3, $4)
RETURNING *