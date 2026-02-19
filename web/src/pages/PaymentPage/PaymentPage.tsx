import { useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import styles from "./PaymentPage.module.scss";
import type { AxiosError } from "axios";
import type { Payment, PaymentResponse } from "@/types/models/Payment";
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

type PaymentPageProps = {
    /** Identifier of the payment to display */
    paymentId: string;
    /** Optional preloaded payment payload for deterministic stories/tests */
    initialPayment?: Payment | null;
};

/**
 * The authenticated payment detail page.
 * Fetches and displays a single payment, including payout and receipt status details.
 *
 * Route: `/payments/$paymentId`
 *
 * ## Props
 *
 * - `paymentId` - Identifier of the payment to fetch and render.
 * - `initialPayment` - Optional preloaded payment payload used by stories/tests.
 *
 * ## Related Components
 *
 * - `BackButton` - Navigates back to the payments list.
 * - `Notification` - Displays not-found and API failure feedback.
 */
function PaymentPage({ paymentId, initialPayment }: PaymentPageProps) {
    const { addNotification } = useNotification();
    const hasInitialPayment = initialPayment !== undefined;
    const hasShownErrorRef = useRef<boolean>(false);
    const {
        data: fetchedPayment,
        isLoading,
        isError,
        isRefetching,
        error,
        refetch,
    } = useQuery<Payment, AxiosError<ApiErrorResponse>>({
        queryKey: ["payment", paymentId],
        queryFn: async () => {
            const response = await api.get<PaymentResponse>(`/payments/${paymentId}`);
            return response.data.payment;
        },
        enabled: !hasInitialPayment,
        staleTime: Number.POSITIVE_INFINITY,
        refetchOnWindowFocus: false,
        retry: false,
    });

    const payment = hasInitialPayment ? initialPayment || null : fetchedPayment || null;
    const isNotFound = isError && error.response?.status === 404;
    const hasApiFailure = isError && !isNotFound;
    const shouldShowNotFoundState = !isLoading && (isNotFound || (!hasApiFailure && !payment));

    useEffect(() => {
        hasShownErrorRef.current = false;
    }, [paymentId]);

    useEffect(() => {
        if (!isError || hasShownErrorRef.current) {
            return;
        }

        hasShownErrorRef.current = true;

        if (error.response?.status === 404) {
            addNotification({
                type: NotificationType.ERROR,
                title: "Payment Not Found",
                message: "Unable to load the requested payment.",
            });
            return;
        }

        addNotification({
            type: NotificationType.ERROR,
            title: "Payment Unavailable",
            message: "Unable to load payment details right now.",
        });
    }, [addNotification, error, isError]);

    const payoutTypeSupportsTransferStatus = (payoutType: Payment["payout_type"]) => {
        return ["paypal", "venmo", "zelle"].includes(payoutType);
    };

    const getPayoutTypeLabel = (payoutType: Payment["payout_type"]) => {
        if (payoutType === "direct_deposit") {
            return "Direct Deposit";
        }

        return `${payoutType.charAt(0).toUpperCase()}${payoutType.slice(1)}`;
    };

    const getDateLabel = (dateValue: string | null) => {
        if (!dateValue) {
            return "Not set";
        }

        const parsedDate = new Date(`${dateValue}T00:00:00`);

        if (Number.isNaN(parsedDate.getTime())) {
            return dateValue;
        }

        return parsedDate.toLocaleDateString();
    };

    const getBooleanLabel = (isTrue: boolean) => {
        return isTrue ? "Yes" : "No";
    };

    return (
        <section className={styles["payment-page"]}>
            {isLoading && <p className={styles["payment-page__state"]}>Loading payment...</p>}

            {shouldShowNotFoundState && (
                <div className={styles["payment-page__state"]}>
                    <p>This payment could not be found.</p>
                </div>
            )}

            {!isLoading && hasApiFailure && (
                <div className={styles["payment-page__state"]}>
                    <p>Unable to load this payment right now.</p>
                    <button
                        className={styles["payment-page__retry-action"]}
                        onClick={() => {
                            void refetch();
                        }}
                        type="button"
                    >
                        {isRefetching ? "Retrying..." : "Retry"}
                    </button>
                </div>
            )}

            {!isLoading && !hasApiFailure && payment && (
                <>
                    <header className={styles["payment-page__header"]}>
                        <div>
                            <p className={styles["payment-page__eyebrow"]}>Payout details</p>
                            <h1>Payment: ${payment.total}</h1>
                        </div>
                        <BackButton href="/payments">Back to Payments</BackButton>
                    </header>

                    <div className={styles["payment-page__summary-grid"]}>
                        <article className={styles["payment-page__summary-card"]}>
                            <p>Payout type</p>
                            <h3>{getPayoutTypeLabel(payment.payout_type)}</h3>
                        </article>
                        <article className={styles["payment-page__summary-card"]}>
                            <p>Expected payout</p>
                            <h3>{getDateLabel(payment.expected_payout_date)}</h3>
                        </article>
                        <article className={styles["payment-page__summary-card"]}>
                            <p>Payment received</p>
                            <h3>{getBooleanLabel(payment.payment_received)}</h3>
                        </article>
                        {payoutTypeSupportsTransferStatus(payment.payout_type) && (
                            <article className={styles["payment-page__summary-card"]}>
                                <p>Expected transfer</p>
                                <h3>{getDateLabel(payment.expected_transfer_date)}</h3>
                            </article>
                        )}
                        {payoutTypeSupportsTransferStatus(payment.payout_type) && (
                            <article className={styles["payment-page__summary-card"]}>
                                <p>Transfer initiated</p>
                                <h3>{getBooleanLabel(payment.transfer_initiated)}</h3>
                            </article>
                        )}
                        {payoutTypeSupportsTransferStatus(payment.payout_type) && (
                            <article className={styles["payment-page__summary-card"]}>
                                <p>Transfer received</p>
                                <h3>{getBooleanLabel(payment.transfer_received)}</h3>
                            </article>
                        )}
                        <article className={styles["payment-page__summary-card"]}>
                            <p>Tax withholdings covered</p>
                            <h3>{getBooleanLabel(payment.tax_withholdings_covered)}</h3>
                        </article>
                    </div>
                </>
            )}
        </section>
    );
}

export default PaymentPage;
