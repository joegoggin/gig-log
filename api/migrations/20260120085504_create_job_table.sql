CREATE TYPE payment_type_enum AS ENUM ('hourly', 'payouts');

CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    payment_type payment_type_enum NOT NULL,
    number_of_payouts INTEGER,
    payout_amount DECIMAL(10, 2),
    hourly_rate DECIMAL(10, 2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Enforce consistency for 'hourly' jobs
    CONSTRAINT check_hourly_consistency CHECK (
        (payment_type = 'hourly' AND hourly_rate IS NOT NULL AND number_of_payouts IS NULL AND payout_amount IS NULL) OR
        payment_type != 'hourly'
    ),

    -- Enforce consistency for 'payouts' jobs
    CONSTRAINT check_payouts_consistency CHECK (
        (payment_type = 'payouts' AND number_of_payouts IS NOT NULL AND payout_amount IS NOT NULL AND hourly_rate IS NULL) OR
        payment_type != 'payouts'
    )
);

CREATE INDEX idx_jobs_company_id ON jobs(company_id);
CREATE INDEX idx_jobs_user_id ON jobs(user_id);
