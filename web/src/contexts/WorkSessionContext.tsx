import {
    createContext,
    useCallback,
    useContext,
    useEffect,
    useState,
} from "react";
import type { ReactNode } from "react";
import api from "@/lib/axios";

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

type WorkSessionResponse = {
    work_session: WorkSession;
};

export type WorkSessionContextValue = {
    activeSession: WorkSession | null;
    isLoading: boolean;
    timerValue: number;
    refreshSession: () => Promise<void>;
    startSession: (jobId: string) => Promise<void>;
    pauseSession: (sessionId: string) => Promise<void>;
    resumeSession: (sessionId: string) => Promise<void>;
    completeSession: (sessionId: string) => Promise<void>;
};

export const WorkSessionContext = createContext<WorkSessionContextValue | null>(
    null,
);

/**
 * Calculates the total elapsed working time in seconds for a given session.
 */
function calculateElapsedSeconds(session: WorkSession): number {
    if (!session.start_time) {
        return 0;
    }

    const start = new Date(session.start_time).getTime();

    if (session.end_time) {
        const end = new Date(session.end_time).getTime();
        const totalMs = end - start;
        const totalSecs = Math.floor(totalMs / 1000);
        return Math.max(0, totalSecs - session.accumulated_paused_duration);
    }

    if (!session.is_running && session.paused_at) {
        const pausedAt = new Date(session.paused_at).getTime();
        const totalMs = pausedAt - start;
        const totalSecs = Math.floor(totalMs / 1000);
        return Math.max(0, totalSecs - session.accumulated_paused_duration);
    }

    const now = new Date().getTime();
    const totalMs = now - start;
    const totalSecs = Math.floor(totalMs / 1000);
    return Math.max(0, totalSecs - session.accumulated_paused_duration);
}

type WorkSessionProviderProps = {
    children: ReactNode;
};

export function WorkSessionProvider({ children }: WorkSessionProviderProps) {
    const [activeSession, setActiveSession] = useState<WorkSession | null>(
        null,
    );
    const [isLoading, setIsLoading] = useState(true);
    const [timerValue, setTimerValue] = useState(0);

    const refreshSession = useCallback(async () => {
        try {
            const response = await api.get<WorkSessionResponse>(
                "/work-sessions/active",
            );
            setActiveSession(response.data.work_session);
        } catch (error: any) {
            if (error.response?.status === 404) {
                setActiveSession(null);
            } else {
                throw error;
            }
        }
    }, []);

    const startSession = useCallback(async (jobId: string) => {
        const response = await api.post<WorkSessionResponse>(
            "/work-sessions/start",
            { job_id: jobId },
        );
        setActiveSession(response.data.work_session);
    }, []);

    const pauseSession = useCallback(async (sessionId: string) => {
        const response = await api.post<WorkSessionResponse>(
            `/work-sessions/${sessionId}/pause`,
        );
        setActiveSession(response.data.work_session);
    }, []);

    const resumeSession = useCallback(async (sessionId: string) => {
        const response = await api.post<WorkSessionResponse>(
            `/work-sessions/${sessionId}/resume`,
        );
        setActiveSession(response.data.work_session);
    }, []);

    const completeSession = useCallback(async (sessionId: string) => {
        await api.post<WorkSessionResponse>(
            `/work-sessions/${sessionId}/complete`,
        );
        setActiveSession(null); // Assuming completing makes it inactive
    }, []);

    useEffect(() => {
        const fetchInitialSession = async () => {
            setIsLoading(true);
            try {
                await refreshSession();
            } catch (error) {
                console.error("Failed to fetch initial work session", error);
            } finally {
                setIsLoading(false);
            }
        };

        void fetchInitialSession();
    }, [refreshSession]);

    // Timer logic
    useEffect(() => {
        if (!activeSession) {
            setTimerValue(0);
            return;
        }

        // Set initial timer immediately
        setTimerValue(calculateElapsedSeconds(activeSession));

        if (!activeSession.is_running) {
            return;
        }

        const intervalId = setInterval(() => {
            setTimerValue(calculateElapsedSeconds(activeSession));
        }, 1000);

        return () => clearInterval(intervalId);
    }, [activeSession]);

    return (
        <WorkSessionContext.Provider
            value={{
                activeSession,
                isLoading,
                timerValue,
                refreshSession,
                startSession,
                pauseSession,
                resumeSession,
                completeSession,
            }}
        >
            {children}
        </WorkSessionContext.Provider>
    );
}

export function useWorkSession() {
    const context = useContext(WorkSessionContext);

    if (!context) {
        throw new Error(
            "useWorkSession must be used within a WorkSessionProvider",
        );
    }

    return context;
}
