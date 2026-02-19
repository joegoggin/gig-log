import { useEffect, useState } from "react";
import styles from "./CreateJobPage.module.scss";
import type { Company } from "@/types/models/Company";
import type { JobPaymentType } from "@/types/models/Job";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import BackButton from "@/components/core/BackButton/BackButton";
import Button from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import TextInput from "@/components/core/TextInput/TextInput";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";

type CreateJobPageProps = {
    /** Optional company identifier used to preselect the company field */
    preselectedCompanyId?: string;
};

type CreateJobFormData = {
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
        targetFields: Array<keyof CreateJobFormData>,
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
 * The authenticated create-job page.
 * Captures job details, supports payment-type-specific inputs, and submits
 * a create request to the API.
 *
 * Route: `/jobs/create`
 *
 * ## Props
 *
 * - `preselectedCompanyId` - Optional company identifier used to preselect the company field.
 *
 * ## Related Components
 *
 * - `Form` - Handles create-job submission.
 * - `TextInput` - Captures title and payment amounts/rates.
 * - `BackButton` - Navigates back to the jobs list.
 * - `Button` - Submits the create-job form.
 */
function CreateJobPage({ preselectedCompanyId }: CreateJobPageProps) {
    const { addNotification } = useNotification();
    const [companies, setCompanies] = useState<Array<Company>>([]);
    const [isLoadingCompanies, setIsLoadingCompanies] = useState<boolean>(true);
    const [hasLoadError, setHasLoadError] = useState<boolean>(false);
    const { data, errors, setData, setErrors } = useForm<CreateJobFormData>({
        title: "",
        company_id: preselectedCompanyId || "",
        payment_type: "hourly",
        number_of_payouts: "",
        payout_amount: "",
        hourly_rate: "",
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
                message: "Unable to load companies for job creation.",
            });
        } finally {
            setIsLoadingCompanies(false);
        }
    };

    useEffect(() => {
        void fetchCompanies();
    }, [addNotification, preselectedCompanyId, setData]);

    const getResetCompanyId = () => {
        if (!preselectedCompanyId) {
            return "";
        }

        const hasPreselectedCompany = companies.some(
            (company) => company.id === preselectedCompanyId,
        );

        return hasPreselectedCompany ? preselectedCompanyId : "";
    };

    const resetCreateJobForm = () => {
        setData("title", "");
        setData("company_id", getResetCompanyId());
        setData("payment_type", "hourly");
        setData("number_of_payouts", "");
        setData("payout_amount", "");
        setData("hourly_rate", "");
        setErrors({});
    };

    const createJobMutation = useFormMutation({
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
                payout_amount:
                    data.payment_type === "payouts" ? data.payout_amount || null : null,
                hourly_rate: data.payment_type === "hourly" ? data.hourly_rate || null : null,
            };

            const response = await api.post("/jobs", payload);
            return response.data;
        },
        onSuccess: () => {
            resetCreateJobForm();
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Job Created",
                message: "Your job has been created successfully.",
            });
        },
        onError: (incomingErrors) => {
            setErrors(mapSchemaErrorsToFields(incomingErrors));
        },
        fallbackError: "Failed to create job",
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

        createJobMutation.mutate();
    };

    return (
        <section className={styles["create-job-page"]}>
            <header className={styles["create-job-page__header"]}>
                <div>
                    <p className={styles["create-job-page__eyebrow"]}>Work setup</p>
                    <h1>Create Job</h1>
                    <p className={styles["create-job-page__lead"]}>
                        Configure this job&apos;s client and payment model before you start tracking
                        sessions.
                    </p>
                </div>
                <BackButton href="/jobs">Back to Jobs</BackButton>
            </header>

            <div className={styles["create-job-page__panel"]}>
                {isLoadingCompanies && (
                    <div className={styles["create-job-page__state"]}>
                        <p>Loading companies...</p>
                    </div>
                )}

                {!isLoadingCompanies && hasLoadError && (
                    <div className={styles["create-job-page__state"]}>
                        <p>Unable to load companies right now.</p>
                        <button
                            className={styles["create-job-page__retry-action"]}
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
                    <div className={styles["create-job-page__state"]}>
                        <p>Create a company first before adding a job.</p>
                        <Button href="/companies/create">Create Company</Button>
                    </div>
                )}

                {!isLoadingCompanies && !hasLoadError && companies.length > 0 && (
                    <Form onSubmit={handleSubmit}>
                        <TextInput
                            name="title"
                            placeholder="Job Title"
                            data={data}
                            setData={setData}
                            errors={errors}
                        />

                        <div
                            className={`${styles["create-job-page__field"]} ${errors.company_id ? styles["create-job-page__field--error"] : ""}`}
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
                            className={`${styles["create-job-page__field"]} ${errors.payment_type ? styles["create-job-page__field--error"] : ""}`}
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

                        <Button type="submit">Create Job</Button>
                    </Form>
                )}
            </div>
        </section>
    );
}

export default CreateJobPage;
