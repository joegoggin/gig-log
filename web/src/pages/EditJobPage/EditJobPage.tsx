import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import styles from "./EditJobPage.module.scss";
import type { Company } from "@/types/models/Company";
import type { JobPaymentType, JobResponse } from "@/types/models/Job";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import BackButton from "@/components/core/BackButton/BackButton";
import Button from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import TextInput from "@/components/core/TextInput/TextInput";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";

type EditJobPageProps = {
    /** Identifier of the job to edit */
    jobId: string;
};

type EditJobFormData = {
    title: string;
    company_id: string;
    payment_type: JobPaymentType;
    number_of_payouts: string;
    payout_amount: string;
    hourly_rate: string;
};

type CompaniesListResponse = {
    companies: Array<Company>;
};

const mapSchemaErrorsToFields = (
    incomingErrors: Record<string, string>,
): Record<string, string> => {
    const mappedErrors = { ...incomingErrors };

    const applySchemaError = (
        schemaField: string,
        targetFields: Array<keyof EditJobFormData>,
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

    applySchemaError("hourly_rate_required", ["hourly_rate"]);
    applySchemaError("hourly_payout_fields_forbidden", ["number_of_payouts", "payout_amount"]);
    applySchemaError("payout_fields_required", ["number_of_payouts", "payout_amount"]);
    applySchemaError("payouts_hourly_rate_forbidden", ["hourly_rate"]);

    return mappedErrors;
};

/**
 * The authenticated edit-job page.
 * Loads existing job details, pre-fills payment-specific fields, and submits
 * updates back to the API.
 *
 * Route: `/jobs/$jobId/edit`
 *
 * ## Props
 *
 * - `jobId` - Identifier of the job to fetch and update.
 *
 * ## Related Components
 *
 * - `Form` - Handles update submission lifecycle.
 * - `TextInput` - Captures title and payment amounts/rates.
 * - `BackButton` - Navigates back to the job detail page.
 * - `Button` - Submits the edit-job form.
 */
function EditJobPage({ jobId }: EditJobPageProps) {
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const { addNotification } = useNotification();
    const [companies, setCompanies] = useState<Array<Company>>([]);
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [hasLoadError, setHasLoadError] = useState<boolean>(false);
    const { data, errors, setData, setErrors } = useForm<EditJobFormData>({
        title: "",
        company_id: "",
        payment_type: "hourly",
        number_of_payouts: "",
        payout_amount: "",
        hourly_rate: "",
    });

    const fetchJobAndCompanies = async () => {
        setIsLoading(true);
        setHasLoadError(false);

        try {
            const [jobResponse, companiesResponse] = await Promise.all([
                api.get<JobResponse>(`/jobs/${jobId}`),
                api.get<CompaniesListResponse>("/companies"),
            ]);
            const { job } = jobResponse.data;
            const fetchedCompanies = companiesResponse.data.companies;
            const companyExists = fetchedCompanies.some((company) => company.id === job.company_id);

            setCompanies(fetchedCompanies);
            setData("title", job.title);
            setData("company_id", companyExists ? job.company_id : "");
            setData("payment_type", job.payment_type);
            setData(
                "number_of_payouts",
                job.number_of_payouts !== null ? String(job.number_of_payouts) : "",
            );
            setData("payout_amount", job.payout_amount || "");
            setData("hourly_rate", job.hourly_rate || "");
        } catch {
            setHasLoadError(true);
            addNotification({
                type: NotificationType.ERROR,
                title: "Job Unavailable",
                message: "Unable to load this job for editing.",
            });
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        void fetchJobAndCompanies();
    }, [addNotification, jobId, setData]);

    const updateJobMutation = useFormMutation({
        mutationFn: async () => {
            const numberOfPayouts = data.number_of_payouts.trim();

            const payload = {
                company_id: data.company_id,
                title: data.title,
                payment_type: data.payment_type,
                number_of_payouts:
                    data.payment_type === "payouts" && numberOfPayouts
                        ? Number(numberOfPayouts)
                        : null,
                payout_amount: data.payment_type === "payouts" ? data.payout_amount || null : null,
                hourly_rate: data.payment_type === "hourly" ? data.hourly_rate || null : null,
            };

            const response = await api.put<JobResponse>(`/jobs/${jobId}`, payload);
            return response.data;
        },
        onSuccess: (response) => {
            queryClient.setQueryData(["job", jobId], response.job);
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Job Updated",
                message: "Job details were updated successfully.",
            });
            navigate({ to: `/jobs/${jobId}` });
        },
        onError: (incomingErrors) => {
            setErrors(mapSchemaErrorsToFields(incomingErrors));
        },
        fallbackError: "Failed to update job",
    });

    const validateForm = () => {
        const nextErrors: Record<string, string> = {};

        if (!data.title.trim()) {
            nextErrors.title = "Job title is required";
        }

        if (!data.company_id) {
            nextErrors.company_id = "Company is required";
        }

        if (data.payment_type === "hourly" && !data.hourly_rate.trim()) {
            nextErrors.hourly_rate = "Hourly rate is required";
        }

        if (data.payment_type === "payouts") {
            if (!data.number_of_payouts.trim()) {
                nextErrors.number_of_payouts = "Number of payouts is required";
            } else {
                const parsedPayoutCount = Number(data.number_of_payouts);

                if (!Number.isInteger(parsedPayoutCount)) {
                    nextErrors.number_of_payouts = "Number of payouts must be a whole number";
                }
            }

            if (!data.payout_amount.trim()) {
                nextErrors.payout_amount = "Payout amount is required";
            }
        }

        return nextErrors;
    };

    const handleSubmit = () => {
        const nextErrors = validateForm();

        if (Object.keys(nextErrors).length > 0) {
            setErrors(nextErrors);
            return;
        }

        updateJobMutation.mutate();
    };

    return (
        <section className={styles["edit-job-page"]}>
            <header className={styles["edit-job-page__header"]}>
                <div>
                    <p className={styles["edit-job-page__eyebrow"]}>Work setup</p>
                    <h1>Edit Job</h1>
                    <p className={styles["edit-job-page__lead"]}>
                        Update this job&apos;s client and payment model to keep your records
                        accurate.
                    </p>
                </div>
                <BackButton href={`/jobs/${jobId}`}>Back to Job</BackButton>
            </header>

            <div className={styles["edit-job-page__panel"]}>
                {isLoading && (
                    <div className={styles["edit-job-page__state"]}>
                        <p>Loading job...</p>
                    </div>
                )}

                {!isLoading && hasLoadError && (
                    <div className={styles["edit-job-page__state"]}>
                        <p>Unable to load this job right now.</p>
                        <button
                            className={styles["edit-job-page__retry-action"]}
                            onClick={() => {
                                void fetchJobAndCompanies();
                            }}
                            type="button"
                        >
                            Retry
                        </button>
                    </div>
                )}

                {!isLoading && !hasLoadError && companies.length === 0 && (
                    <div className={styles["edit-job-page__state"]}>
                        <p>Create a company first before updating this job.</p>
                        <Button href="/companies/create">Create Company</Button>
                    </div>
                )}

                {!isLoading && !hasLoadError && companies.length > 0 && (
                    <Form onSubmit={handleSubmit}>
                        <TextInput
                            name="title"
                            placeholder="Job Title"
                            data={data}
                            setData={setData}
                            errors={errors}
                        />

                        <div
                            className={`${styles["edit-job-page__field"]} ${errors.company_id ? styles["edit-job-page__field--error"] : ""}`}
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
                            {errors.company_id && <p>{errors.company_id}</p>}
                        </div>

                        <div
                            className={`${styles["edit-job-page__field"]} ${errors.payment_type ? styles["edit-job-page__field--error"] : ""}`}
                        >
                            <label htmlFor="payment_type">Payment Type</label>
                            <select
                                id="payment_type"
                                onChange={(event) => {
                                    setData("payment_type", event.target.value as JobPaymentType);
                                }}
                                value={data.payment_type}
                            >
                                <option value="hourly">Hourly</option>
                                <option value="payouts">Payouts</option>
                            </select>
                            {errors.payment_type && <p>{errors.payment_type}</p>}
                        </div>

                        {data.payment_type === "hourly" && (
                            <TextInput
                                name="hourly_rate"
                                placeholder="Hourly Rate"
                                type="number"
                                data={data}
                                setData={setData}
                                errors={errors}
                            />
                        )}

                        {data.payment_type === "payouts" && (
                            <>
                                <TextInput
                                    name="number_of_payouts"
                                    placeholder="Number of Payouts"
                                    type="number"
                                    data={data}
                                    setData={setData}
                                    errors={errors}
                                />
                                <TextInput
                                    name="payout_amount"
                                    placeholder="Payout Amount"
                                    type="number"
                                    data={data}
                                    setData={setData}
                                    errors={errors}
                                />
                            </>
                        )}

                        <Button type="submit">Save Job</Button>
                    </Form>
                )}
            </div>
        </section>
    );
}

export default EditJobPage;
