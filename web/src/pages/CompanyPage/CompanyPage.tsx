import { useEffect, useRef, useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import styles from "./CompanyPage.module.scss";
import type {
    CompanyDetailResponse,
    CompanyJob,
    CompanyPayment,
} from "@/types/models/Company";
import AddIcon from "@/components/icons/AddIcon";
import DeleteIcon from "@/components/icons/DeleteIcon";
import EditIcon from "@/components/icons/EditIcon";
import InfoIcon from "@/components/icons/InfoIcon";
import BackButton from "@/components/core/BackButton/BackButton";
import Button, { ButtonVariant } from "@/components/core/Button/Button";
import { NotificationType } from "@/components/core/Notification/Notification";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";

type CompanyPageProps = {
    /** Identifier of the company to display */
    companyId: string;
    /** Optional preloaded company-detail response for deterministic stories/tests */
    initialCompanyDetail?: CompanyDetailResponse | null;
};

/**
 * The authenticated company detail page.
 * Displays a company summary plus paginated jobs and payments.
 *
 * Route: `/companies/$companyId`
 *
 * ## Props
 *
 * - `companyId` - Identifier of the company to fetch and render.
 * - `initialCompanyDetail` - Optional preloaded detail payload used by stories/tests.
 *
 * ## Related Components
 *
 * - `BackButton` - Navigates back to the companies list.
 * - `Button` - Handles list pagination actions.
 */
function CompanyPage({ companyId, initialCompanyDetail }: CompanyPageProps) {
    const navigate = useNavigate();
    const { addNotification } = useNotification();
    const hasInitialCompanyDetail = initialCompanyDetail !== undefined;
    const hasShownErrorRef = useRef<boolean>(false);
    const {
        data: fetchedCompanyDetail,
        isLoading,
        isError,
    } = useQuery({
        queryKey: ["company", companyId],
        queryFn: async () => {
            const response = await api.get<CompanyDetailResponse>(`/companies/${companyId}`);
            return response.data;
        },
        enabled: !hasInitialCompanyDetail,
        staleTime: Number.POSITIVE_INFINITY,
        refetchOnWindowFocus: false,
        retry: false,
    });

    const companyDetail = hasInitialCompanyDetail
        ? initialCompanyDetail || null
        : fetchedCompanyDetail || null;

    const [jobs, setJobs] = useState<Array<CompanyJob>>(companyDetail?.paginated_jobs || []);
    const [jobsHasMore, setJobsHasMore] = useState<boolean>(companyDetail?.jobs_has_more || false);
    const [jobsPage, setJobsPage] = useState<number>(1);
    const [payments, setPayments] = useState<Array<CompanyPayment>>(
        companyDetail?.paginated_payments || [],
    );
    const [paymentsHasMore, setPaymentsHasMore] = useState<boolean>(
        companyDetail?.payments_has_more || false,
    );
    const [paymentsPage, setPaymentsPage] = useState<number>(1);
    const [isLoadingMoreJobs, setIsLoadingMoreJobs] = useState<boolean>(false);
    const [isLoadingMorePayments, setIsLoadingMorePayments] = useState<boolean>(false);

    useEffect(() => {
        setJobs(companyDetail?.paginated_jobs || []);
        setJobsHasMore(companyDetail?.jobs_has_more || false);
        setJobsPage(1);
        setPayments(companyDetail?.paginated_payments || []);
        setPaymentsHasMore(companyDetail?.payments_has_more || false);
        setPaymentsPage(1);
    }, [companyDetail]);

    useEffect(() => {
        hasShownErrorRef.current = false;
    }, [companyId]);

    useEffect(() => {
        if (isError && !hasShownErrorRef.current) {
            hasShownErrorRef.current = true;
            addNotification({
                type: NotificationType.ERROR,
                title: "Company Not Found",
                message: "Unable to load the requested company.",
            });
        }
    }, [addNotification, isError]);

    const getReceivedString = (isReceived: boolean) => {
        return isReceived ? "Received" : "Not Received";
    };

    const paymentRequiresTransferStatus = (payoutType: string) => {
        return payoutType === "paypal" || payoutType === "venmo" || payoutType === "zelle";
    };

    const loadMoreJobs = async () => {
        if (!companyDetail || !jobsHasMore || isLoadingMoreJobs) {
            return;
        }

        const nextPage = jobsPage + 1;
        setIsLoadingMoreJobs(true);

        try {
            const response = await api.get<CompanyDetailResponse>(
                `/companies/${companyId}?jobs_page=${nextPage}`,
            );
            setJobs((currentJobs) => [...currentJobs, ...response.data.paginated_jobs]);
            setJobsHasMore(response.data.jobs_has_more);
            setJobsPage(nextPage);
        } catch {
            addNotification({
                type: NotificationType.ERROR,
                title: "Jobs Unavailable",
                message: "Unable to load more jobs right now.",
            });
        } finally {
            setIsLoadingMoreJobs(false);
        }
    };

    const loadMorePayments = async () => {
        if (!companyDetail || !paymentsHasMore || isLoadingMorePayments) {
            return;
        }

        const nextPage = paymentsPage + 1;
        setIsLoadingMorePayments(true);

        try {
            const response = await api.get<CompanyDetailResponse>(
                `/companies/${companyId}?payments_page=${nextPage}`,
            );
            setPayments((currentPayments) => [
                ...currentPayments,
                ...response.data.paginated_payments,
            ]);
            setPaymentsHasMore(response.data.payments_has_more);
            setPaymentsPage(nextPage);
        } catch {
            addNotification({
                type: NotificationType.ERROR,
                title: "Payments Unavailable",
                message: "Unable to load more payments right now.",
            });
        } finally {
            setIsLoadingMorePayments(false);
        }
    };

    return (
        <section className={styles["company-page"]}>
            {isLoading && <p>Loading company...</p>}

            {!isLoading && !companyDetail && <p>This company could not be found.</p>}

            {!isLoading && companyDetail && (
                <>
                    <div className={styles["company-page__title-bar"]}>
                        <div>
                            <p className={styles["company-page__eyebrow"]}>Company details</p>
                            <h1>{companyDetail.company.name}</h1>
                        </div>
                        <BackButton href="/companies">Back to Companies</BackButton>
                    </div>

                    <div className={styles["company-page__summary"]}>
                        {companyDetail.company.requires_tax_withholdings && (
                            <article className={styles["company-page__summary-card"]}>
                                <p>Tax withholding</p>
                                <h3>{companyDetail.company.tax_withholding_rate}%</h3>
                            </article>
                        )}
                        <article className={styles["company-page__summary-card"]}>
                            <p>Total payments</p>
                            <h3>${companyDetail.company.payment_total}</h3>
                        </article>
                        <article className={styles["company-page__summary-card"]}>
                            <p>Total hours</p>
                            <h3>{companyDetail.company.hours}</h3>
                        </article>
                    </div>

                    <div className={styles["company-page__lists"]}>
                        <section className={styles["company-page__list"]}>
                            <div className={styles["company-page__list-title-bar"]}>
                                <h2>Jobs</h2>
                                <button
                                    className={`${styles["company-page__icon-button"]} ${styles["company-page__icon-button--add"]}`}
                                    onClick={() => {
                                        navigate({ to: "/jobs" });
                                    }}
                                    type="button"
                                >
                                    <AddIcon />
                                    <p>Add Job</p>
                                </button>
                            </div>

                            {jobs.length > 0 ? (
                                jobs.map((job) => (
                                    <article key={job.id} className={styles["company-page__list-item"]}>
                                        <div className={styles["company-page__list-content"]}>
                                            <h3>{job.title}</h3>
                                        </div>
                                        <div className={styles["company-page__list-actions"]}>
                                            <button
                                                aria-label={`View ${job.title}`}
                                                className={`${styles["company-page__icon-button"]} ${styles["company-page__icon-button--view"]}`}
                                                onClick={() => {
                                                    navigate({ to: `/jobs/${job.id}` });
                                                }}
                                                type="button"
                                            >
                                                <InfoIcon />
                                            </button>
                                            <button
                                                aria-disabled="true"
                                                aria-label="Edit job action (coming soon)"
                                                className={`${styles["company-page__icon-button"]} ${styles["company-page__icon-button--edit"]} ${styles["company-page__icon-button--disabled"]}`}
                                                tabIndex={-1}
                                                type="button"
                                            >
                                                <EditIcon />
                                            </button>
                                            <button
                                                aria-disabled="true"
                                                aria-label="Delete job action (coming soon)"
                                                className={`${styles["company-page__icon-button"]} ${styles["company-page__icon-button--delete"]} ${styles["company-page__icon-button--disabled"]}`}
                                                tabIndex={-1}
                                                type="button"
                                            >
                                                <DeleteIcon />
                                            </button>
                                        </div>
                                    </article>
                                ))
                            ) : (
                                <h3>No Jobs</h3>
                            )}

                            {jobsHasMore && (
                                <Button
                                    onClick={() => {
                                        void loadMoreJobs();
                                    }}
                                    variant={ButtonVariant.SECONDARY}
                                >
                                    {isLoadingMoreJobs ? "Loading Jobs..." : "Load More Jobs"}
                                </Button>
                            )}
                        </section>

                        <section className={styles["company-page__list"]}>
                            <div className={styles["company-page__list-title-bar"]}>
                                <h2>Payments</h2>
                                <button
                                    className={`${styles["company-page__icon-button"]} ${styles["company-page__icon-button--add"]}`}
                                    onClick={() => {
                                        navigate({ to: "/payments" });
                                    }}
                                    type="button"
                                >
                                    <AddIcon />
                                    <p>Add Payment</p>
                                </button>
                            </div>

                            {payments.length > 0 ? (
                                payments.map((payment) => (
                                    <article
                                        key={payment.id}
                                        className={styles["company-page__list-item"]}
                                    >
                                        <div className={styles["company-page__list-content"]}>
                                            <h3>Total: ${payment.total}</h3>
                                            <h3>Payout Type: {payment.payout_type}</h3>
                                            <h3>
                                                Payment: {getReceivedString(payment.payment_received)}
                                            </h3>
                                            {paymentRequiresTransferStatus(payment.payout_type) && (
                                                <h3>
                                                    Transfer: {getReceivedString(payment.transfer_received)}
                                                </h3>
                                            )}
                                        </div>
                                        <div className={styles["company-page__list-actions"]}>
                                            <button
                                                aria-disabled="true"
                                                aria-label="View payment action (coming soon)"
                                                className={`${styles["company-page__icon-button"]} ${styles["company-page__icon-button--view"]} ${styles["company-page__icon-button--disabled"]}`}
                                                tabIndex={-1}
                                                type="button"
                                            >
                                                <InfoIcon />
                                            </button>
                                            <button
                                                aria-disabled="true"
                                                aria-label="Edit payment action (coming soon)"
                                                className={`${styles["company-page__icon-button"]} ${styles["company-page__icon-button--edit"]} ${styles["company-page__icon-button--disabled"]}`}
                                                tabIndex={-1}
                                                type="button"
                                            >
                                                <EditIcon />
                                            </button>
                                            <button
                                                aria-disabled="true"
                                                aria-label="Delete payment action (coming soon)"
                                                className={`${styles["company-page__icon-button"]} ${styles["company-page__icon-button--delete"]} ${styles["company-page__icon-button--disabled"]}`}
                                                tabIndex={-1}
                                                type="button"
                                            >
                                                <DeleteIcon />
                                            </button>
                                        </div>
                                    </article>
                                ))
                            ) : (
                                <h3>No Payments</h3>
                            )}

                            {paymentsHasMore && (
                                <Button
                                    onClick={() => {
                                        void loadMorePayments();
                                    }}
                                    variant={ButtonVariant.SECONDARY}
                                >
                                    {isLoadingMorePayments
                                        ? "Loading Payments..."
                                        : "Load More Payments"}
                                </Button>
                            )}
                        </section>
                    </div>
                </>
            )}
        </section>
    );
}

export default CompanyPage;
