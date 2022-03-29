INSERT INTO affiliate_managers (
    affiliate_id,
    user_id,
    create_time,
    update_time
  )
VALUES ($1, $2, $3, $4)
RETURNING *