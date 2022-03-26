INSERT INTO nonprofits (
    nonprofit_id,
    create_time,
    update_time,
    change_nonprofit_id,
    icon_url,
    name,
    ein,
    mission,
    category,
    affiliate_id
  )
VALUES (DEFAULT, $1, $2, $3, $4, $5, $6, $7, $8, $9)
RETURNING *