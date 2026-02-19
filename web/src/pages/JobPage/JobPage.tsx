import { useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import styles from "./JobPage.module.scss";
import type { AxiosError } from "axios";
import type { Job, JobResponse } from "@/types/models/Job";
import BackButton from "@/components/core/BackButton/BackButton";
import { NotificationType } from "@/components/core/Notification/Notification";
import { useNotification } from "@/contexts/NotificationContext";
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
 *
 * ## Related Components
 *
 * - `BackButton` - Navigates back to the jobs list.
 * - `Notification` - Displays not-found and API failure feedback.
 */
function JobPage({ jobId, initialJob }: JobPageProps) {
    const { addNotification } = useNotification();
    const hasInitialJob = initialJob !== undefined;
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

    const job = hasInitialJob ? initialJob || null : fetchedJob || null;
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
                </>
            )}
        </section>
    );
}

export default JobPage;
