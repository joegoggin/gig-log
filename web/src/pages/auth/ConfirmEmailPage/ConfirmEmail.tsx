import { useNavigate } from "@tanstack/react-router";
import styles from "./ConfirmEmail.module.scss";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import Button from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import TextInput from "@/components/core/TextInput/TextInput";
import { useNotification } from "@/contexts/NotificationContext";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import api from "@/lib/axios";

type ConfirmEmailPageProps = {
    /** Email address that should be confirmed */
    email?: string;
};

type ConfirmEmailFormData = {
    auth_code: string;
};

type ConfirmEmailResponse = {
    message: string;
};

/**
 * The email confirmation page for newly registered accounts.
 * Validates a confirmation code sent to the provided email address.
 *
 * Route: `/auth/confirm-email`
 *
 * ## Props
 *
 * - `email` - Email address associated with the confirmation code
 *
 * ## Related Components
 *
 * - `Form` - Handles confirmation-code submission.
 * - `TextInput` - Captures the email confirmation code.
 * - `Button` - Submits the confirmation request.
 * - `FullscreenCenteredLayout` - Centers page content.
 */
function ConfirmEmailPage({ email }: ConfirmEmailPageProps) {
    const navigate = useNavigate();
    const { addNotification } = useNotification();
    const { data, errors, setData, setErrors } = useForm<ConfirmEmailFormData>({
        auth_code: "",
    });

    const confirmEmailMutation = useFormMutation({
        mutationFn: async () => {
            const response = await api.post<ConfirmEmailResponse>(
                "/auth/confirm-email",
                {
                    email,
                    auth_code: data.auth_code,
                },
            );
            return response.data;
        },
        onSuccess: () => {
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Email Confirmed",
                message: "Your email has been confirmed. You can now log in.",
            });
            navigate({ to: "/auth/log-in" });
        },
        onError: setErrors,
        fallbackError: "Failed to confirm email",
    });

    const onSubmit = () => {
        if (!email) {
            setErrors({ auth_code: "Email is required" });
            return;
        }
        confirmEmailMutation.mutate();
    };

    return (
        <FullscreenCenteredLayout className={styles["confirm-email"]}>
            <h1>Confirm Email</h1>
            <Form onSubmit={onSubmit}>
                <TextInput
                    name="auth_code"
                    placeholder="Enter confirmation code"
                    data={data}
                    setData={setData}
                    errors={errors}
                />
                <Button type="submit">Confirm Email</Button>
            </Form>
        </FullscreenCenteredLayout>
    );
}

export default ConfirmEmailPage;
