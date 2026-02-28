CREATE TABLE work_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    start_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    end_time TIMESTAMPTZ,
    is_running BOOLEAN NOT NULL DEFAULT true,
    accumulated_paused_duration INTERVAL NOT NULL DEFAULT interval '0',
    paused_at TIMESTAMPTZ,
    time_reported INTERVAL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_running_no_end_time CHECK (is_running != true OR end_time IS NULL),
    CONSTRAINT chk_paused_must_be_running CHECK (paused_at IS NULL OR is_running = true)
);

