import { useEffect, useMemo, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import styles from "./JobPage.module.scss";
import type { AxiosError } from "axios";
import type { Job, JobResponse } from "@/types/models/Job";
import type { WorkSession, WorkSessionListResponse } from "@/types/models/WorkSession";
import type {TimerStatus} from "@/components/work-sessions/WorkSessionTimer";
import BackButton from "@/components/core/BackButton/BackButton";
import { NotificationType } from "@/components/core/Notification/Notification";
import { useNotification } from "@/contexts/NotificationContext";
import { useWorkSession } from "@/contexts/WorkSessionContext";
import WorkSessionTimer from "@/components/work-sessions/WorkSessionTimer";
import api from "@/lib/axios";

type ApiErrorResponse = {
    error?: {
        code?: string;
        message?: string;
    };
};

type JobPageProps = {
    /** Identifier of the job to display */
    jobId: string;
    /** Optional preloaded job payload for deterministic stories/tests */
    initialJob?: Job | null;
    /** Optional preloaded work sessions payload for deterministic stories/tests */
    initialWorkSessions?: Array<WorkSession>;
};

/**
 * The authenticated job detail page.
 * Fetches and displays a single job, including payment-model details.
 *
 * Route: `/jobs/$jobId`
 *
 * ## Props
 *
 * - `jobId` - Identifier of the job to fetch and render.
 * - `initialJob` - Optional preloaded job payload used by stories/tests.
 * - `initialWorkSessions` - Optional preloaded work sessions used by stories/tests.
 *
 * ## Related Components
 *
 * - `BackButton` - Navigates back to the jobs list.
 * - `Notification` - Displays not-found and API failure feedback.
 * - `WorkSessionTimer` - Controls time tracking for the job.
 */
function JobPage({ jobId, initialJob, initialWorkSessions }: JobPageProps) {
    const { addNotification } = useNotification();
    const {
        activeSession,
        startSession,
        pauseSession,
        resumeSession,
        completeSession,
    } = useWorkSession();

    const hasInitialJob = initialJob !== undefined;
    const hasInitialWorkSessions = initialWorkSessions !== undefined;
    const hasShownErrorRef = useRef<boolean>(false);
    const {
        data: fetchedJob,
        isLoading,
        isError,
        isRefetching,
        error,
        refetch,
    } = useQuery<Job, AxiosError<ApiErrorResponse>>({
        queryKey: ["job", jobId],
        queryFn: async () => {
            const response = await api.get<JobResponse>(`/jobs/${jobId}`);
            return response.data.job;
        },
        enabled: !hasInitialJob,
        staleTime: Number.POSITIVE_INFINITY,
        refetchOnWindowFocus: false,
        retry: false,
    });

    const {
        data: fetchedWorkSessions,
        isLoading: isWorkSessionsLoading,
        refetch: refetchWorkSessions,
    } = useQuery<Array<WorkSession>, AxiosError<ApiErrorResponse>>({
        queryKey: ["work-sessions", jobId],
        queryFn: async () => {
            const response = await api.get<WorkSessionListResponse>(
                `/jobs/${jobId}/work-sessions`,
            );
            return response.data.work_sessions;
        },
        enabled: !hasInitialWorkSessions,
    });

    const job = hasInitialJob ? initialJob || null : fetchedJob || null;
    const workSessions = hasInitialWorkSessions
        ? initialWorkSessions
        : fetchedWorkSessions || [];

    const isNotFound = isError && error.response?.status === 404;
    const hasApiFailure = isError && !isNotFound;
    const shouldShowNotFoundState = !isLoading && (isNotFound || (!hasApiFailure && !job));

    useEffect(() => {
        hasShownErrorRef.current = false;
    }, [jobId]);

    useEffect(() => {
        if (!isError || hasShownErrorRef.current) {
            return;
        }

        hasShownErrorRef.current = true;

        if (error.response?.status === 404) {
            addNotification({
                type: NotificationType.ERROR,
                title: "Job Not Found",
                message: "Unable to load the requested job.",
            });
            return;
        }

        addNotification({
            type: NotificationType.ERROR,
            title: "Job Unavailable",
            message: "Unable to load job details right now.",
        });
    }, [addNotification, error, isError]);

    const getPaymentTypeLabel = (paymentType: Job["payment_type"]) => {
        return paymentType === "hourly" ? "Hourly" : "Payouts";
    };

    const getPayoutSummary = (selectedJob: Job) => {
        const hasPayoutDetails =
            selectedJob.number_of_payouts !== null && selectedJob.payout_amount !== null;

        if (!hasPayoutDetails) {
            return "Payout details not provided";
        }

        return `${selectedJob.number_of_payouts} payout${selectedJob.number_of_payouts === 1 ? "" : "s"} at $${selectedJob.payout_amount}`;
    };

    const isThisJobActive = activeSession?.job_id === jobId;
    const isAnotherJobActive = activeSession !== null && !isThisJobActive;

    const timerStatus: TimerStatus = useMemo(() => {
        if (!activeSession) return "idle";
        if (isAnotherJobActive) return "idle";
        if (activeSession.end_time) return "completed";
        if (!activeSession.is_running) return "paused";
        return "running";
    }, [activeSession, isAnotherJobActive]);

    const handleStart = async () => {
        if (!jobId) return;
        try {
            await startSession(jobId);
        } catch (err: any) {
            addNotification({
                type: NotificationType.ERROR,
                title: "Failed to Start",
                message: err.response?.data?.error?.message || "Could not start work session.",
            });
        }
    };

    const handlePause = async () => {
        if (!activeSession) return;
        try {
            await pauseSession(activeSession.id);
        } catch (err: any) {
            addNotification({
                type: NotificationType.ERROR,
                title: "Failed to Pause",
                message: err.response?.data?.error?.message || "Could not pause work session.",
            });
        }
    };

    const handleResume = async () => {
        if (!activeSession) return;
        try {
            await resumeSession(activeSession.id);
        } catch (err: any) {
            addNotification({
                type: NotificationType.ERROR,
                title: "Failed to Resume",
                message: err.response?.data?.error?.message || "Could not resume work session.",
            });
        }
    };

    const handleComplete = async () => {
        if (!activeSession) return;
        try {
            await completeSession(activeSession.id);
            void refetchWorkSessions();
        } catch (err: any) {
            addNotification({
                type: NotificationType.ERROR,
                title: "Failed to Complete",
                message: err.response?.data?.error?.message || "Could not complete work session.",
            });
        }
    };

    const getAccumulatedSeconds = () => {
        if (!activeSession || !activeSession.start_time) return 0;
        
        if (activeSession.is_running) {
            return -activeSession.accumulated_paused_duration;
        } else {
            const start = new Date(activeSession.start_time).getTime();
            const pausedAt = activeSession.paused_at 
                ? new Date(activeSession.paused_at).getTime() 
                : new Date().getTime();
            const totalMs = pausedAt - start;
            return Math.max(0, Math.floor(totalMs / 1000) - activeSession.accumulated_paused_duration);
        }
    };

    const historicalSessions = workSessions.filter((s) => s.end_time !== null);

    return (
        <section className={styles["job-page"]}>
            {isLoading && <p className={styles["job-page__state"]}>Loading job...</p>}

            {shouldShowNotFoundState && (
                <div className={styles["job-page__state"]}>
                    <p>This job could not be found.</p>
                </div>
            )}

            {!isLoading && hasApiFailure && (
                <div className={styles["job-page__state"]}>
                    <p>Unable to load this job right now.</p>
                    <button
                        className={styles["job-page__retry-action"]}
                        onClick={() => {
                            void refetch();
                        }}
                        type="button"
                    >
                        {isRefetching ? "Retrying..." : "Retry"}
                    </button>
                </div>
            )}

            {!isLoading && !hasApiFailure && job && (
                <>
                    <header className={styles["job-page__header"]}>
                        <div>
                            <p className={styles["job-page__eyebrow"]}>Work details</p>
                            <h1>{job.title}</h1>
                        </div>
                        <BackButton href="/jobs">Back to Jobs</BackButton>
                    </header>

                    <div className={styles["job-page__summary-grid"]}>
                        <article className={styles["job-page__summary-card"]}>
                            <p>Payment type</p>
                            <h3>{getPaymentTypeLabel(job.payment_type)}</h3>
                        </article>
                        <article className={styles["job-page__summary-card"]}>
                            <p>Company ID</p>
                            <h3>{job.company_id}</h3>
                        </article>
                        <article className={styles["job-page__summary-card"]}>
                            <p>Created</p>
                            <h3>{new Date(job.created_at).toLocaleDateString()}</h3>
                        </article>
                    </div>

                    <section className={styles["job-page__details"]}>
                        <h2>Payment model details</h2>

                        {job.payment_type === "hourly" ? (
                            <p>
                                Hourly rate: {job.hourly_rate ? `$${job.hourly_rate}/hour` : "Not provided"}
                            </p>
                        ) : (
                            <p>{getPayoutSummary(job)}</p>
                        )}
                    </section>

                    <section className={styles["job-page__sessions-section"]}>
                        <div className={styles["job-page__sessions-header"]}>
                            <h2>Time Tracking</h2>
                            {isAnotherJobActive && (
                                <p className={styles["job-page__sessions-warning"]}>
                                    You have a running work session for another job. Complete it
                                    first to track time here.
                                </p>
                            )}
                        </div>

                        {!isAnotherJobActive && (
                            <div className={styles["job-page__timer-wrapper"]}>
                                <WorkSessionTimer
                                    status={timerStatus}
                                    startTime={
                                        activeSession?.is_running && activeSession.start_time
                                            ? activeSession.start_time
                                            : null
                                    }
                                    accumulatedSeconds={getAccumulatedSeconds()}
                                    onStart={handleStart}
                                    onPause={handlePause}
                                    onResume={handleResume}
                                    onComplete={handleComplete}
                                />
                            </div>
                        )}

                        <div className={styles["job-page__historical-sessions"]}>
                            <h3>Work History</h3>
                            {isWorkSessionsLoading ? (
                                <p>Loading history...</p>
                            ) : historicalSessions.length === 0 ? (
                                <p className={styles["job-page__empty-sessions"]}>
                                    No completed work sessions yet.
                                </p>
                            ) : (
                                <ul className={styles["job-page__sessions-list"]}>
                                    {historicalSessions.map((session) => {
                                        const start = new Date(session.start_time as string).getTime();
                                        const end = new Date(session.end_time as string).getTime();
                                        const totalSecs = (end - start) / 1000 - session.accumulated_paused_duration;
                                        const hours = Math.max(0, totalSecs / 3600);

                                        return (
                                            <li key={session.id} className={styles["job-page__session-item"]}>
                                                <div>
                                                    <strong>
                                                        {new Date(session.start_time as string).toLocaleDateString()}
                                                    </strong>
                                                    <span>
                                                        {new Date(session.start_time as string).toLocaleTimeString()} -{" "}
                                                        {new Date(session.end_time as string).toLocaleTimeString()}
                                                    </span>
                                                </div>
                                                <div className={styles["job-page__session-hours"]}>
                                                    {hours.toFixed(2)} hrs
                                                </div>
                                            </li>
                                        );
                                    })}
                                </ul>
                            )}
                        </div>
                    </section>
                </>
            )}
        </section>
    );
}

export default JobPage;
