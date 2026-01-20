CREATE TABLE companies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    requires_tax_withholdings BOOLEAN NOT NULL DEFAULT FALSE,
    tax_withholding_rate DECIMAL(5, 2)
);

CREATE INDEX idx_companies_user_id ON companies(user_id);
