export type WorkSession = {
    id: string;
    user_id: string;
    job_id: string;
    start_time: string | null;
    end_time: string | null;
    is_running: boolean;
    accumulated_paused_duration: number;
    paused_at: string | null;
    time_reported: boolean;
    created_at: string;
    updated_at: string;
};

export type WorkSessionResponse = {
    work_session: WorkSession;
};

export type WorkSessionListResponse = {
    work_sessions: Array<WorkSession>;
};
