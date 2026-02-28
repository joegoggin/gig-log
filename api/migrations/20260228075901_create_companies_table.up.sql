CREATE TABLE companies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    requires_tax_withholdings BOOLEAN NOT NULL DEFAULT false,
    tax_withholding_rate DECIMAL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_tax_withholding_rate_presence CHECK (
        (requires_tax_withholdings = true AND tax_withholding_rate IS NOT NULL)
        OR
        (requires_tax_withholdings = false AND tax_withholding_rate IS NULL)
    )
);
