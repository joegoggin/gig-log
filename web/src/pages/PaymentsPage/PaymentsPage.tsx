import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import styles from "./PaymentsPage.module.scss";
import type {
    DeletePaymentResponse,
    Payment,
    PaymentsListResponse,
} from "@/types/models/Payment";
import AddIcon from "@/components/icons/AddIcon";
import DeleteIcon from "@/components/icons/DeleteIcon";
import EditIcon from "@/components/icons/EditIcon";
import InfoIcon from "@/components/icons/InfoIcon";
import { NotificationType } from "@/components/core/Notification/Notification";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";

type PaymentsPageProps = {
    /** Optional preloaded payments for deterministic rendering in stories/tests */
    initialPayments?: Array<Payment>;
};

/**
 * The authenticated payments index page.
 * Displays all payments owned by the current user and provides actions
 * for create/view/edit/delete workflows.
 *
 * Route: `/payments`
 *
 * ## Props
 *
 * - `initialPayments` - Optional preloaded payment records used by stories/tests.
 *
 * ## Related Components
 *
 * - `MainLayout` - Wraps the page with primary app navigation.
 * - `Notification` - Displays feedback for fetch/delete operations.
 */
function PaymentsPage({ initialPayments }: PaymentsPageProps) {
    const navigate = useNavigate();
    const { addNotification } = useNotification();
    const hasInitialPayments = initialPayments !== undefined;
    const [payments, setPayments] = useState<Array<Payment>>(initialPayments || []);
    const [isLoading, setIsLoading] = useState<boolean>(!hasInitialPayments);
    const [hasLoadError, setHasLoadError] = useState<boolean>(false);
    const [deletingPaymentId, setDeletingPaymentId] = useState<string | null>(null);

    const fetchPayments = useCallback(async () => {
        setIsLoading(true);
        setHasLoadError(false);

        try {
            const response = await api.get<PaymentsListResponse>("/payments");
            setPayments(response.data.payments);
        } catch {
            setHasLoadError(true);
            addNotification({
                type: NotificationType.ERROR,
                title: "Payments Unavailable",
                message: "Failed to load payments.",
            });
        } finally {
            setIsLoading(false);
        }
    }, [addNotification]);

    useEffect(() => {
        if (hasInitialPayments) {
            return;
        }

        void fetchPayments();
    }, [fetchPayments, hasInitialPayments]);

    const getPayoutTypeLabel = (payoutType: Payment["payout_type"]) => {
        if (payoutType === "direct_deposit") {
            return "Direct Deposit";
        }

        if (payoutType === "paypal") {
            return "PayPal";
        }

        return `${payoutType.charAt(0).toUpperCase()}${payoutType.slice(1)}`;
    };

    const payoutTypeUsesTransferStatus = (payoutType: Payment["payout_type"]) => {
        return ["paypal", "venmo", "zelle"].includes(payoutType);
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

    const handleDeletePayment = async (payment: Payment) => {
        const shouldDelete = window.confirm(
            `Delete this $${payment.total} payment? This cannot be undone.`,
        );

        if (!shouldDelete || deletingPaymentId) {
            return;
        }

        setDeletingPaymentId(payment.id);

        try {
            await api.delete<DeletePaymentResponse>(`/payments/${payment.id}`);
            setPayments((currentPayments) =>
                currentPayments.filter((currentPayment) => currentPayment.id !== payment.id),
            );
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Payment Deleted",
                message: "Payment was deleted successfully.",
            });
        } catch {
            addNotification({
                type: NotificationType.ERROR,
                title: "Delete Failed",
                message: "Unable to delete this payment right now.",
            });
        } finally {
            setDeletingPaymentId(null);
        }
    };

    return (
        <section className={styles["payments-page"]}>
            <header className={styles["payments-page__header"]}>
                <div>
                    <p className={styles["payments-page__eyebrow"]}>Payout records</p>
                    <h1>Payments</h1>
                    <p className={styles["payments-page__lead"]}>
                        Review incoming payouts and manage payment records.
                    </p>
                </div>
                <button
                    className={`${styles["payments-page__icon-button"]} ${styles["payments-page__create-action"]}`}
                    onClick={() => {
                        navigate({ to: "/payments/create" as never });
                    }}
                    type="button"
                >
                    <AddIcon />
                    <p>Create Payment</p>
                </button>
            </header>

            {isLoading && <div className={styles["payments-page__state"]}>Loading payments...</div>}

            {!isLoading && hasLoadError && (
                <div className={styles["payments-page__state"]}>
                    <p>Unable to load payments right now.</p>
                    <button
                        className={`${styles["payments-page__icon-button"]} ${styles["payments-page__retry-action"]}`}
                        onClick={() => {
                            void fetchPayments();
                        }}
                        type="button"
                    >
                        <p>Retry</p>
                    </button>
                </div>
            )}

            {!isLoading && !hasLoadError && payments.length === 0 && (
                <div className={styles["payments-page__state"]}>
                    <p>No payments yet. Create your first payment to start tracking payouts.</p>
                </div>
            )}

            {!isLoading && !hasLoadError && payments.length > 0 && (
                <div className={styles["payments-page__grid"]}>
                    {payments.map((payment) => (
                        <article key={payment.id} className={styles["payments-page__payment-card"]}>
                            <div>
                                <h3>Total: ${payment.total}</h3>
                                <p className={styles["payments-page__meta"]}>
                                    Payout type: {getPayoutTypeLabel(payment.payout_type)}
                                </p>
                                <p className={styles["payments-page__meta"]}>
                                    Expected payout: {getDateLabel(payment.expected_payout_date)}
                                </p>
                                {payoutTypeUsesTransferStatus(payment.payout_type) && (
                                    <p className={styles["payments-page__meta"]}>
                                        Expected transfer: {getDateLabel(payment.expected_transfer_date)}
                                    </p>
                                )}
                                <p className={styles["payments-page__meta"]}>
                                    Payment received: {getBooleanLabel(payment.payment_received)}
                                </p>
                                {payoutTypeUsesTransferStatus(payment.payout_type) && (
                                    <p className={styles["payments-page__meta"]}>
                                        Transfer received: {getBooleanLabel(payment.transfer_received)}
                                    </p>
                                )}
                                <p className={styles["payments-page__meta"]}>
                                    Tax withholdings covered: {getBooleanLabel(payment.tax_withholdings_covered)}
                                </p>
                            </div>
                            <div className={styles["payments-page__actions"]}>
                                <button
                                    className={`${styles["payments-page__icon-button"]} ${styles["payments-page__view-action"]}`}
                                    onClick={() => {
                                        navigate({ to: `/payments/${payment.id}` as never });
                                    }}
                                    type="button"
                                >
                                    <InfoIcon />
                                    <p>View Payment</p>
                                </button>
                                <button
                                    className={`${styles["payments-page__icon-button"]} ${styles["payments-page__edit-action"]}`}
                                    onClick={() => {
                                        navigate({ to: `/payments/${payment.id}/edit` as never });
                                    }}
                                    type="button"
                                >
                                    <EditIcon />
                                    <p>Edit Payment</p>
                                </button>
                                <button
                                    className={`${styles["payments-page__icon-button"]} ${styles["payments-page__delete-action"]}`}
                                    onClick={() => {
                                        void handleDeletePayment(payment);
                                    }}
                                    type="button"
                                >
                                    <DeleteIcon />
                                    <p>
                                        {deletingPaymentId === payment.id
                                            ? "Deleting Payment..."
                                            : "Delete Payment"}
                                    </p>
                                </button>
                            </div>
                        </article>
                    ))}
                </div>
            )}
        </section>
    );
}

export default PaymentsPage;
