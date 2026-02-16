import { useNavigate } from "@tanstack/react-router";
import styles from "./CreateCompanyPage.module.scss";
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

type CreateCompanyFormData = {
    name: string;
    requires_tax_withholdings: boolean;
    tax_withholding_rate: string;
};

/**
 * The authenticated create-company page.
 * Captures company details and submits a create request to the API.
 *
 * Route: `/companies/create`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `Form` - Handles create-company submission.
 * - `TextInput` - Captures company name and optional tax rate.
 * - `CheckBox` - Toggles tax-withholding behavior.
 * - `BackButton` - Navigates back to the companies list.
 * - `Button` - Submits the create-company form.
 */
function CreateCompanyPage() {
    const navigate = useNavigate();
    const { addNotification } = useNotification();
    const { data, errors, setData, setErrors } = useForm<CreateCompanyFormData>({
        name: "",
        requires_tax_withholdings: false,
        tax_withholding_rate: "",
    });

    const createCompanyMutation = useFormMutation({
        mutationFn: async () => {
            const payload = {
                name: data.name,
                requires_tax_withholdings: data.requires_tax_withholdings,
                tax_withholding_rate: data.requires_tax_withholdings
                    ? data.tax_withholding_rate || null
                    : null,
            };

            const response = await api.post("/companies", payload);
            return response.data;
        },
        onSuccess: () => {
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Company Created",
                message: "Your company has been created successfully.",
            });
            navigate({ to: "/companies" });
        },
        onError: setErrors,
        fallbackError: "Failed to create company",
    });

    const handleSubmit = () => {
        createCompanyMutation.mutate();
    };

    return (
        <section className={styles["create-company-page"]}>
            <header className={styles["create-company-page__header"]}>
                <h1>Create Company</h1>
                <BackButton href="/companies">Back to Companies</BackButton>
            </header>

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
                <Button type="submit">Create Company</Button>
            </Form>
        </section>
    );
}

export default CreateCompanyPage;
