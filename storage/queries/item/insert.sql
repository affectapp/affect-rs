INSERT INTO items (
    item_id,
    create_time,
    update_time,
    user_id,
    plaid_item_id,
    plaid_access_token
  )
VALUES (DEFAULT, $1, $2, $3, $4, $5)
RETURNING *