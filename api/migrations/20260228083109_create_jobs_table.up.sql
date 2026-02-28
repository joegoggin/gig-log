CREATE TYPE payment_type AS ENUM ('hourly', 'payouts');

CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR NOT NULL,
    payment_type payment_type NOT NULL,
    hourly_rate DECIMAL,
    number_of_payouts INTEGER,
    payout_amount DECIMAL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_payment_fields_consistent CHECK (
        (payment_type = 'hourly'
         AND hourly_rate IS NOT NULL
         AND number_of_payouts IS NULL
         AND payout_amount IS NULL)
        OR
        (payment_type = 'payouts'
         AND hourly_rate IS NULL
         AND number_of_payouts IS NOT NULL
         AND payout_amount IS NOT NULL)
    )
);
