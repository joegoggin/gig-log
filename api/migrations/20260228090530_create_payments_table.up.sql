CREATE TYPE payout_type AS ENUM ('paypal', 'cash', 'check', 'zelle', 'venmo', 'direct_deposit');

CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    total DECIMAL NOT NULL,
    payout_type payout_type NOT NULL,
    expected_payout_date DATE,
    transfer_initiated BOOLEAN NOT NULL DEFAULT false,
    payment_received BOOLEAN NOT NULL DEFAULT false,
    tax_withholdings_covered BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
