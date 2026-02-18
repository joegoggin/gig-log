import { useCallback, useEffect, useState } from "react";
import styles from "./JobsPage.module.scss";
import type { Job, JobsListResponse } from "@/types/models/Job";
import AddIcon from "@/components/icons/AddIcon";
import DeleteIcon from "@/components/icons/DeleteIcon";
import EditIcon from "@/components/icons/EditIcon";
import InfoIcon from "@/components/icons/InfoIcon";
import { NotificationType } from "@/components/core/Notification/Notification";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";

type JobsPageProps = {
    /** Optional preloaded jobs for deterministic rendering in stories/tests */
    initialJobs?: Array<Job>;
};

/**
 * The authenticated jobs index page.
 * Displays all jobs owned by the current user and provides actions
 * for upcoming create/view/edit flows plus immediate delete support.
 *
 * Route: `/jobs`
 *
 * ## Props
 *
 * - `initialJobs` - Optional preloaded job records used by stories/tests.
 *
 * ## Related Components
 *
 * - `MainLayout` - Wraps the page with primary app navigation.
 * - `Notification` - Displays feedback for fetch/delete operations.
 */
function JobsPage({ initialJobs }: JobsPageProps) {
    const { addNotification } = useNotification();
    const hasInitialJobs = initialJobs !== undefined;
    const [jobs, setJobs] = useState<Array<Job>>(initialJobs || []);
    const [isLoading, setIsLoading] = useState<boolean>(!hasInitialJobs);
    const [hasLoadError, setHasLoadError] = useState<boolean>(false);
    const [deletingJobId, setDeletingJobId] = useState<string | null>(null);

    const fetchJobs = useCallback(async () => {
        setIsLoading(true);
        setHasLoadError(false);

        try {
            const response = await api.get<JobsListResponse>("/jobs");
            setJobs(response.data.jobs);
        } catch {
            setHasLoadError(true);
            addNotification({
                type: NotificationType.ERROR,
                title: "Jobs Unavailable",
                message: "Failed to load jobs.",
            });
        } finally {
            setIsLoading(false);
        }
    }, [addNotification]);

    useEffect(() => {
        if (hasInitialJobs) {
            return;
        }

        void fetchJobs();
    }, [fetchJobs, hasInitialJobs]);

    const getPaymentTypeLabel = (paymentType: Job["payment_type"]) => {
        return paymentType === "hourly" ? "Hourly" : "Payouts";
    };

    const getPaymentDetails = (job: Job) => {
        if (job.payment_type === "hourly") {
            return job.hourly_rate ? `Rate: $${job.hourly_rate}/hour` : "Rate not provided";
        }

        const hasPayoutDetails = job.number_of_payouts !== null && job.payout_amount !== null;

        if (hasPayoutDetails) {
            return `${job.number_of_payouts} payout${job.number_of_payouts === 1 ? "" : "s"} at $${job.payout_amount}`;
        }

        return "Payout details not provided";
    };

    const handleDeleteJob = async (job: Job) => {
        const shouldDelete = window.confirm(
            `Delete "${job.title}"? This also removes related work sessions and payments.`,
        );

        if (!shouldDelete || deletingJobId) {
            return;
        }

        setDeletingJobId(job.id);

        try {
            await api.delete(`/jobs/${job.id}`);
            setJobs((currentJobs) => currentJobs.filter((currentJob) => currentJob.id !== job.id));
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Job Deleted",
                message: `${job.title} was deleted successfully.`,
            });
        } catch {
            addNotification({
                type: NotificationType.ERROR,
                title: "Delete Failed",
                message: "Unable to delete this job right now.",
            });
        } finally {
            setDeletingJobId(null);
        }
    };

    return (
        <section className={styles["jobs-page"]}>
            <header className={styles["jobs-page__header"]}>
                <div>
                    <p className={styles["jobs-page__eyebrow"]}>Work records</p>
                    <h1>Jobs</h1>
                    <p className={styles["jobs-page__lead"]}>
                        Review active work and clean up completed jobs.
                    </p>
                </div>
                <button
                    aria-disabled="true"
                    className={`${styles["jobs-page__icon-button"]} ${styles["jobs-page__create-action"]} ${styles["jobs-page__icon-button--disabled"]}`}
                    tabIndex={-1}
                    type="button"
                >
                    <AddIcon />
                    <p>Create Job (coming soon)</p>
                </button>
            </header>

            {isLoading && <div className={styles["jobs-page__state"]}>Loading jobs...</div>}

            {!isLoading && hasLoadError && (
                <div className={styles["jobs-page__state"]}>
                    <p>Unable to load jobs right now.</p>
                    <button
                        className={`${styles["jobs-page__icon-button"]} ${styles["jobs-page__retry-action"]}`}
                        onClick={() => {
                            void fetchJobs();
                        }}
                        type="button"
                    >
                        <p>Retry</p>
                    </button>
                </div>
            )}

            {!isLoading && !hasLoadError && jobs.length === 0 && (
                <div className={styles["jobs-page__state"]}>
                    <p>No jobs yet. Create your first job to start tracking work.</p>
                </div>
            )}

            {!isLoading && !hasLoadError && jobs.length > 0 && (
                <div className={styles["jobs-page__grid"]}>
                    {jobs.map((job) => (
                        <article key={job.id} className={styles["jobs-page__job-card"]}>
                            <div>
                                <h3>{job.title}</h3>
                                <p className={styles["jobs-page__meta"]}>
                                    Payment type: {getPaymentTypeLabel(job.payment_type)}
                                </p>
                                <p className={styles["jobs-page__meta"]}>{getPaymentDetails(job)}</p>
                            </div>
                            <div className={styles["jobs-page__actions"]}>
                                <button
                                    aria-disabled="true"
                                    className={`${styles["jobs-page__icon-button"]} ${styles["jobs-page__view-action"]} ${styles["jobs-page__icon-button--disabled"]}`}
                                    tabIndex={-1}
                                    type="button"
                                >
                                    <InfoIcon />
                                    <p>View Job</p>
                                </button>
                                <button
                                    aria-disabled="true"
                                    className={`${styles["jobs-page__icon-button"]} ${styles["jobs-page__edit-action"]} ${styles["jobs-page__icon-button--disabled"]}`}
                                    tabIndex={-1}
                                    type="button"
                                >
                                    <EditIcon />
                                    <p>Edit Job</p>
                                </button>
                                <button
                                    className={`${styles["jobs-page__icon-button"]} ${styles["jobs-page__delete-action"]}`}
                                    onClick={() => {
                                        void handleDeleteJob(job);
                                    }}
                                    type="button"
                                >
                                    <DeleteIcon />
                                    <p>{deletingJobId === job.id ? "Deleting Job..." : "Delete Job"}</p>
                                </button>
                            </div>
                        </article>
                    ))}
                </div>
            )}
        </section>
    );
}

export default JobsPage;
