CREATE TYPE payout_type_enum AS ENUM ('paypal', 'cash', 'check', 'zelle', 'venmo', 'direct_deposit');

CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    total DECIMAL(10, 2) NOT NULL,
    payout_type payout_type_enum NOT NULL,
    expected_payout_date DATE,
    expected_transfer_date DATE,
    transfer_initiated BOOLEAN NOT NULL DEFAULT FALSE,
    payment_received BOOLEAN NOT NULL DEFAULT FALSE,
    transfer_received BOOLEAN NOT NULL DEFAULT FALSE,
    tax_withholdings_covered BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payments_user_id ON payments(user_id);
CREATE INDEX idx_payments_company_id ON payments(company_id);
