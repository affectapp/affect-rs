INSERT INTO accounts (
    account_id,
    create_time,
    update_time,
    item_id,
    plaid_account_id,
    name,
    mask,
    stripe_bank_account_id
  )
VALUES (DEFAULT, $1, $2, $3, $4, $5, $6, $7)
RETURNING *