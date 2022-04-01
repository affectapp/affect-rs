ALTER TABLE affiliates
ADD CONSTRAINT fk_affiliate_to_nonprofit FOREIGN KEY (asserted_nonprofit_id) REFERENCES nonprofits(nonprofit_id);