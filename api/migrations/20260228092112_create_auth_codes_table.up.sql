CREATE TYPE code_type AS ENUM ('email_verification', 'password_reset', 'email_change');

CREATE TABLE auth_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code VARCHAR NOT NULL,
    code_type code_type NOT NULL,
    new_email VARCHAR,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_new_email_for_email_change CHECK (
        (code_type = 'email_change' AND new_email IS NOT NULL)
        OR
        (code_type != 'email_change' AND new_email IS NULL)
    )
);
