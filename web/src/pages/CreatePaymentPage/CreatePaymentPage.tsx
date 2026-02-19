import { useEffect, useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import styles from "./CreatePaymentPage.module.scss";
import type { Company } from "@/types/models/Company";
import type { PaymentPayoutType, PaymentResponse } from "@/types/models/Payment";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import BackButton from "@/components/core/BackButton/BackButton";
import Button from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";

type CreatePaymentPageProps = {
    /** Optional company identifier used to preselect the company field */
    preselectedCompanyId?: string;
};

type CreatePaymentFormData = {
    company_id: string;
    total: string;
    payout_type: PaymentPayoutType;
    expected_payout_date: string;
    expected_transfer_date: string;
    transfer_initiated: boolean;
    payment_received: boolean;
    transfer_received: boolean;
    tax_withholdings_covered: boolean;
};

type CompaniesListResponse = {
    companies: Array<Company>;
};

const payoutTypeOptions: Array<{ value: PaymentPayoutType; label: string }> = [
    { value: "paypal", label: "PayPal" },
    { value: "cash", label: "Cash" },
    { value: "check", label: "Check" },
    { value: "zelle", label: "Zelle" },
    { value: "venmo", label: "Venmo" },
    { value: "direct_deposit", label: "Direct Deposit" },
];

const payoutTypeSupportsTransferStatus = (payoutType: PaymentPayoutType) => {
    return ["paypal", "venmo", "zelle"].includes(payoutType);
};

const mapSchemaErrorsToFields = (
    incomingErrors: Record<string, string>,
): Record<string, string> => {
    const mappedErrors = { ...incomingErrors };

    const applySchemaError = (
        schemaField: string,
        targetFields: Array<keyof CreatePaymentFormData>,
    ) => {
        const message = incomingErrors[schemaField];

        if (!message) {
            return;
        }

        for (const targetField of targetFields) {
            mappedErrors[targetField] = message;
        }

        delete mappedErrors[schemaField];
    };

    applySchemaError("total_range", ["total"]);
    applySchemaError("expected_transfer_date_forbidden", ["expected_transfer_date"]);
    applySchemaError("transfer_initiated_forbidden", ["transfer_initiated"]);
    applySchemaError("transfer_received_forbidden", ["transfer_received"]);
    applySchemaError("expected_transfer_date_required", ["expected_transfer_date"]);
    applySchemaError("transfer_initiated_requires_payment_received", [
        "transfer_initiated",
        "payment_received",
    ]);
    applySchemaError("transfer_received_requires_transfer_initiated", [
        "transfer_received",
        "transfer_initiated",
    ]);
    applySchemaError("transfer_received_requires_payment_received", [
        "transfer_received",
        "payment_received",
    ]);
    applySchemaError("expected_transfer_date_order", [
        "expected_payout_date",
        "expected_transfer_date",
    ]);

    return mappedErrors;
};

/**
 * The authenticated create-payment page.
 * Captures payout details, validates transfer/date combinations, and submits
 * a create request to the API.
 *
 * Route: `/payments/create`
 *
 * ## Props
 *
 * - `preselectedCompanyId` - Optional company identifier used to preselect the company field.
 *
 * ## Related Components
 *
 * - `Form` - Handles create-payment submission.
 * - `BackButton` - Navigates back to the payments list.
 * - `Button` - Submits the create-payment form and links to create-company flow.
 * - `Notification` - Displays success and error feedback.
 */
function CreatePaymentPage({ preselectedCompanyId }: CreatePaymentPageProps) {
    const navigate = useNavigate();
    const { addNotification } = useNotification();
    const [companies, setCompanies] = useState<Array<Company>>([]);
    const [isLoadingCompanies, setIsLoadingCompanies] = useState<boolean>(true);
    const [hasLoadError, setHasLoadError] = useState<boolean>(false);
    const { data, errors, setData, setErrors } = useForm<CreatePaymentFormData>({
        company_id: preselectedCompanyId || "",
        total: "",
        payout_type: "cash",
        expected_payout_date: "",
        expected_transfer_date: "",
        transfer_initiated: false,
        payment_received: false,
        transfer_received: false,
        tax_withholdings_covered: false,
    });

    const fetchCompanies = async () => {
        setIsLoadingCompanies(true);
        setHasLoadError(false);

        try {
            const response = await api.get<CompaniesListResponse>("/companies");
            const fetchedCompanies = response.data.companies;

            setCompanies(fetchedCompanies);

            if (preselectedCompanyId) {
                const companyExists = fetchedCompanies.some(
                    (company) => company.id === preselectedCompanyId,
                );

                setData("company_id", companyExists ? preselectedCompanyId : "");
            }
        } catch {
            setHasLoadError(true);
            addNotification({
                type: NotificationType.ERROR,
                title: "Companies Unavailable",
                message: "Unable to load companies for payment creation.",
            });
        } finally {
            setIsLoadingCompanies(false);
        }
    };

    useEffect(() => {
        void fetchCompanies();
    }, [addNotification, preselectedCompanyId, setData]);

    const createPaymentMutation = useFormMutation({
        mutationFn: async () => {
            const supportsTransferStatus = payoutTypeSupportsTransferStatus(data.payout_type);

            const payload = {
                company_id: data.company_id,
                total: data.total.trim(),
                payout_type: data.payout_type,
                expected_payout_date: data.expected_payout_date || null,
                expected_transfer_date: supportsTransferStatus
                    ? data.expected_transfer_date || null
                    : null,
                transfer_initiated: supportsTransferStatus ? data.transfer_initiated : false,
                payment_received: data.payment_received,
                transfer_received: supportsTransferStatus ? data.transfer_received : false,
                tax_withholdings_covered: data.tax_withholdings_covered,
            };

            const response = await api.post<PaymentResponse>("/payments", payload);
            return response.data;
        },
        onSuccess: (response) => {
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Payment Created",
                message: "Your payment has been created successfully.",
            });
            navigate({ to: `/payments/${response.payment.id}` as never });
        },
        onError: (incomingErrors) => {
            setErrors(mapSchemaErrorsToFields(incomingErrors));
        },
        fallbackError: "Failed to create payment",
    });

    const validateForm = () => {
        const nextErrors: Record<string, string> = {};
        const supportsTransferStatus = payoutTypeSupportsTransferStatus(data.payout_type);

        if (!data.company_id) {
            nextErrors.company_id = "Company is required";
        }

        if (!data.total.trim()) {
            nextErrors.total = "Total is required";
        } else {
            const parsedTotal = Number(data.total);

            if (Number.isNaN(parsedTotal) || parsedTotal <= 0) {
                nextErrors.total = "Total must be greater than 0";
            }
        }

        if (supportsTransferStatus) {
            if ((data.transfer_initiated || data.transfer_received) && !data.expected_transfer_date) {
                nextErrors.expected_transfer_date =
                    "Expected transfer date is required when transfer status is set";
            }

            if (data.transfer_initiated && !data.payment_received) {
                nextErrors.transfer_initiated =
                    "Transfer initiated requires payment received to be enabled";
                nextErrors.payment_received =
                    "Payment received is required when transfer initiated is enabled";
            }

            if (data.transfer_received && !data.transfer_initiated) {
                nextErrors.transfer_received =
                    "Transfer received requires transfer initiated to be enabled";
                nextErrors.transfer_initiated =
                    "Transfer initiated is required when transfer received is enabled";
            }

            if (data.transfer_received && !data.payment_received) {
                nextErrors.transfer_received =
                    "Transfer received requires payment received to be enabled";
                nextErrors.payment_received =
                    "Payment received is required when transfer received is enabled";
            }
        }

        if (
            data.expected_payout_date &&
            data.expected_transfer_date &&
            data.expected_transfer_date < data.expected_payout_date
        ) {
            nextErrors.expected_transfer_date =
                "Expected transfer date cannot be earlier than expected payout date";
        }

        return nextErrors;
    };

    const handleSubmit = () => {
        if (createPaymentMutation.isPending) {
            return;
        }

        const nextErrors = validateForm();

        if (Object.keys(nextErrors).length > 0) {
            setErrors(nextErrors);
            return;
        }

        createPaymentMutation.mutate();
    };

    const supportsTransferStatus = payoutTypeSupportsTransferStatus(data.payout_type);

    return (
        <section className={styles["create-payment-page"]}>
            <header className={styles["create-payment-page__header"]}>
                <div>
                    <p className={styles["create-payment-page__eyebrow"]}>Payout setup</p>
                    <h1>Create Payment</h1>
                    <p className={styles["create-payment-page__lead"]}>
                        Record a payout with expected dates and receipt status so payment tracking
                        stays accurate.
                    </p>
                </div>
                <BackButton href="/payments">Back to Payments</BackButton>
            </header>

            <div className={styles["create-payment-page__panel"]}>
                {isLoadingCompanies && (
                    <div className={styles["create-payment-page__state"]}>
                        <p>Loading companies...</p>
                    </div>
                )}

                {!isLoadingCompanies && hasLoadError && (
                    <div className={styles["create-payment-page__state"]}>
                        <p>Unable to load companies right now.</p>
                        <button
                            className={styles["create-payment-page__retry-action"]}
                            onClick={() => {
                                void fetchCompanies();
                            }}
                            type="button"
                        >
                            Retry
                        </button>
                    </div>
                )}

                {!isLoadingCompanies && !hasLoadError && companies.length === 0 && (
                    <div className={styles["create-payment-page__state"]}>
                        <p>Create a company first before adding a payment.</p>
                        <Button href="/companies/create">Create Company</Button>
                    </div>
                )}

                {!isLoadingCompanies && !hasLoadError && companies.length > 0 && (
                    <Form onSubmit={handleSubmit}>
                        <div
                            className={`${styles["create-payment-page__field"]} ${errors.company_id ? styles["create-payment-page__field--error"] : ""}`}
                        >
                            <label htmlFor="company_id">Company</label>
                            <select
                                id="company_id"
                                onChange={(event) => {
                                    setData("company_id", event.target.value);
                                }}
                                value={data.company_id}
                            >
                                <option value="">Select Company</option>
                                {companies.map((company) => (
                                    <option key={company.id} value={company.id}>
                                        {company.name}
                                    </option>
                                ))}
                            </select>
                            {errors.company_id && (
                                <p className={styles["create-payment-page__field-error"]}>
                                    {errors.company_id}
                                </p>
                            )}
                        </div>

                        <div
                            className={`${styles["create-payment-page__field"]} ${errors.total ? styles["create-payment-page__field--error"] : ""}`}
                        >
                            <label htmlFor="total">Total</label>
                            <input
                                id="total"
                                min="0"
                                onChange={(event) => {
                                    setData("total", event.target.value);
                                }}
                                placeholder="Total"
                                step="0.01"
                                type="number"
                                value={data.total}
                            />
                            {errors.total && (
                                <p className={styles["create-payment-page__field-error"]}>
                                    {errors.total}
                                </p>
                            )}
                        </div>

                        <div
                            className={`${styles["create-payment-page__field"]} ${errors.payout_type ? styles["create-payment-page__field--error"] : ""}`}
                        >
                            <label htmlFor="payout_type">Payout Type</label>
                            <select
                                id="payout_type"
                                onChange={(event) => {
                                    setData("payout_type", event.target.value as PaymentPayoutType);
                                }}
                                value={data.payout_type}
                            >
                                {payoutTypeOptions.map((payoutTypeOption) => (
                                    <option key={payoutTypeOption.value} value={payoutTypeOption.value}>
                                        {payoutTypeOption.label}
                                    </option>
                                ))}
                            </select>
                            {errors.payout_type && (
                                <p className={styles["create-payment-page__field-error"]}>
                                    {errors.payout_type}
                                </p>
                            )}
                        </div>

                        <div
                            className={`${styles["create-payment-page__field"]} ${errors.expected_payout_date ? styles["create-payment-page__field--error"] : ""}`}
                        >
                            <label htmlFor="expected_payout_date">Expected Payout Date</label>
                            <input
                                id="expected_payout_date"
                                onChange={(event) => {
                                    setData("expected_payout_date", event.target.value);
                                }}
                                type="date"
                                value={data.expected_payout_date}
                            />
                            {errors.expected_payout_date && (
                                <p className={styles["create-payment-page__field-error"]}>
                                    {errors.expected_payout_date}
                                </p>
                            )}
                        </div>

                        {supportsTransferStatus && (
                            <div
                                className={`${styles["create-payment-page__field"]} ${errors.expected_transfer_date ? styles["create-payment-page__field--error"] : ""}`}
                            >
                                <label htmlFor="expected_transfer_date">Expected Transfer Date</label>
                                <input
                                    id="expected_transfer_date"
                                    onChange={(event) => {
                                        setData("expected_transfer_date", event.target.value);
                                    }}
                                    type="date"
                                    value={data.expected_transfer_date}
                                />
                                {errors.expected_transfer_date && (
                                    <p className={styles["create-payment-page__field-error"]}>
                                        {errors.expected_transfer_date}
                                    </p>
                                )}
                            </div>
                        )}

                        <div className={styles["create-payment-page__checkbox-grid"]}>
                            <div>
                                <div
                                    className={`${styles["create-payment-page__checkbox-field"]} ${errors.payment_received ? styles["create-payment-page__checkbox-field--error"] : ""}`}
                                >
                                    <input
                                        checked={data.payment_received}
                                        id="payment_received"
                                        onChange={(event) => {
                                            setData("payment_received", event.target.checked);
                                        }}
                                        type="checkbox"
                                    />
                                    <label htmlFor="payment_received">Payment Received</label>
                                </div>
                                {errors.payment_received && (
                                    <p className={styles["create-payment-page__field-error"]}>
                                        {errors.payment_received}
                                    </p>
                                )}
                            </div>

                            {supportsTransferStatus && (
                                <div>
                                    <div
                                        className={`${styles["create-payment-page__checkbox-field"]} ${errors.transfer_initiated ? styles["create-payment-page__checkbox-field--error"] : ""}`}
                                    >
                                        <input
                                            checked={data.transfer_initiated}
                                            id="transfer_initiated"
                                            onChange={(event) => {
                                                setData("transfer_initiated", event.target.checked);
                                            }}
                                            type="checkbox"
                                        />
                                        <label htmlFor="transfer_initiated">Transfer Initiated</label>
                                    </div>
                                    {errors.transfer_initiated && (
                                        <p className={styles["create-payment-page__field-error"]}>
                                            {errors.transfer_initiated}
                                        </p>
                                    )}
                                </div>
                            )}

                            {supportsTransferStatus && (
                                <div>
                                    <div
                                        className={`${styles["create-payment-page__checkbox-field"]} ${errors.transfer_received ? styles["create-payment-page__checkbox-field--error"] : ""}`}
                                    >
                                        <input
                                            checked={data.transfer_received}
                                            id="transfer_received"
                                            onChange={(event) => {
                                                setData("transfer_received", event.target.checked);
                                            }}
                                            type="checkbox"
                                        />
                                        <label htmlFor="transfer_received">Transfer Received</label>
                                    </div>
                                    {errors.transfer_received && (
                                        <p className={styles["create-payment-page__field-error"]}>
                                            {errors.transfer_received}
                                        </p>
                                    )}
                                </div>
                            )}

                            <div>
                                <div
                                    className={`${styles["create-payment-page__checkbox-field"]} ${errors.tax_withholdings_covered ? styles["create-payment-page__checkbox-field--error"] : ""}`}
                                >
                                    <input
                                        checked={data.tax_withholdings_covered}
                                        id="tax_withholdings_covered"
                                        onChange={(event) => {
                                            setData("tax_withholdings_covered", event.target.checked);
                                        }}
                                        type="checkbox"
                                    />
                                    <label htmlFor="tax_withholdings_covered">
                                        Tax Withholdings Covered
                                    </label>
                                </div>
                                {errors.tax_withholdings_covered && (
                                    <p className={styles["create-payment-page__field-error"]}>
                                        {errors.tax_withholdings_covered}
                                    </p>
                                )}
                            </div>
                        </div>

                        <Button type="submit">
                            {createPaymentMutation.isPending ? "Creating Payment..." : "Create Payment"}
                        </Button>
                    </Form>
                )}
            </div>
        </section>
    );
}

export default CreatePaymentPage;
