import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import styles from "./EditCompanyPage.module.scss";
import type { Company, CompanyDetailResponse } from "@/types/models/Company";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import BackButton from "@/components/core/BackButton/BackButton";
import Button from "@/components/core/Button/Button";
import CheckBox from "@/components/core/CheckBox/CheckBox";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import TextInput from "@/components/core/TextInput/TextInput";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";

type EditCompanyPageProps = {
    /** Identifier of the company to edit */
    companyId: string;
};

type EditCompanyFormData = {
    name: string;
    requires_tax_withholdings: boolean;
    tax_withholding_rate: string;
};

type CompanyResponse = {
    company: Company;
};

/**
 * The authenticated company edit page.
 * Loads existing company data, pre-fills the form, and submits updates.
 *
 * Route: `/companies/$companyId/edit`
 *
 * ## Props
 *
 * - `companyId` - Identifier of the company to fetch and update.
 *
 * ## Related Components
 *
 * - `Form` - Handles update submission lifecycle.
 * - `TextInput` - Captures company name and optional tax rate.
 * - `CheckBox` - Toggles tax-withholding behavior.
 * - `Button` - Submits the update and navigates back.
 */
function EditCompanyPage({ companyId }: EditCompanyPageProps) {
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const { addNotification } = useNotification();
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const { data, errors, setData, setErrors } = useForm<EditCompanyFormData>({
        name: "",
        requires_tax_withholdings: false,
        tax_withholding_rate: "",
    });

    useEffect(() => {
        const fetchCompany = async () => {
            try {
                const response = await api.get<CompanyResponse>(`/companies/${companyId}`);
                const { company } = response.data;

                setData("name", company.name);
                setData("requires_tax_withholdings", company.requires_tax_withholdings);
                setData("tax_withholding_rate", company.tax_withholding_rate || "");
            } catch {
                addNotification({
                    type: NotificationType.ERROR,
                    title: "Company Not Found",
                    message: "Unable to load this company for editing.",
                });
            } finally {
                setIsLoading(false);
            }
        };

        fetchCompany();
    }, [addNotification, companyId, setData]);

    const editCompanyMutation = useFormMutation({
        mutationFn: async () => {
            const payload = {
                name: data.name,
                requires_tax_withholdings: data.requires_tax_withholdings,
                tax_withholding_rate: data.requires_tax_withholdings
                    ? data.tax_withholding_rate || null
                    : null,
            };

            const response = await api.put<CompanyResponse>(`/companies/${companyId}`, payload);
            return response.data;
        },
        onSuccess: (response) => {
            queryClient.setQueryData(["company", companyId], (current) => {
                if (!current || typeof current !== "object" || !("company" in current)) {
                    return current;
                }

                const detail = current as CompanyDetailResponse;

                return {
                    ...detail,
                    company: {
                        ...detail.company,
                        ...response.company,
                    },
                };
            });
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Company Updated",
                message: "Company details were updated successfully.",
            });
            navigate({ to: `/companies/${companyId}` });
        },
        onError: setErrors,
        fallbackError: "Failed to update company",
    });

    const handleSubmit = () => {
        editCompanyMutation.mutate();
    };

    return (
        <section className={styles["edit-company-page"]}>
            <header className={styles["edit-company-page__header"]}>
                <h1>Edit Company</h1>
                <BackButton href={`/companies/${companyId}`}>Back to Company</BackButton>
            </header>

            {isLoading && <p>Loading company...</p>}

            {!isLoading && (
                <Form onSubmit={handleSubmit}>
                    <TextInput
                        name="name"
                        placeholder="Company Name"
                        data={data}
                        setData={setData}
                        errors={errors}
                    />
                    <CheckBox
                        name="requires_tax_withholdings"
                        label="Requires Tax Withholdings"
                        data={data}
                        setData={setData}
                    />
                    {data.requires_tax_withholdings && (
                        <TextInput
                            name="tax_withholding_rate"
                            placeholder="Tax Withholding Rate"
                            data={data}
                            setData={setData}
                            errors={errors}
                        />
                    )}
                    <Button type="submit">Save Company</Button>
                </Form>
            )}
        </section>
    );
}

export default EditCompanyPage;
