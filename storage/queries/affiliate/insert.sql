INSERT INTO affiliates (
                affiliate_id,
                create_time,
                update_time,
                stripe_account_id,
                company_name,
                contact_email,
                business_type,
                asserted_nonprofit_id
        )
VALUES (DEFAULT, $1, $2, $3, $4, $5, $6, $7)
RETURNING affiliate_id,
        create_time,
        update_time,
        stripe_account_id,
        company_name,
        contact_email,
        business_type as "business_type: _",
        asserted_nonprofit_id