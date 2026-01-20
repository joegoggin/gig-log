CREATE TABLE work_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    
    start_time TIMESTAMPTZ,
    end_time TIMESTAMPTZ,
    
    is_running BOOLEAN NOT NULL DEFAULT FALSE,
    accumulated_paused_duration BIGINT NOT NULL DEFAULT 0,
    paused_at TIMESTAMPTZ,
    time_reported BOOLEAN NOT NULL DEFAULT FALSE,

    -- Sanity check: If ended, it must have started, and end > start
    CONSTRAINT check_dates CHECK (
        end_time IS NULL OR 
        (start_time IS NOT NULL AND end_time > start_time)
    ),
    
    -- Consistency: If running, it must have started and not ended
    CONSTRAINT check_running_status CHECK (
        NOT is_running OR 
        (is_running AND start_time IS NOT NULL AND end_time IS NULL)
    )
);

CREATE INDEX idx_work_sessions_user_id ON work_sessions(user_id);
CREATE INDEX idx_work_sessions_job_id ON work_sessions(job_id);
