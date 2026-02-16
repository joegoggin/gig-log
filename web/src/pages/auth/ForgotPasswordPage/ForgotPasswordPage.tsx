import { useNavigate } from "@tanstack/react-router";
import styles from "./ForgotPasswordPage.module.scss";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import Button from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import TextInput from "@/components/core/TextInput/TextInput";
import { useNotification } from "@/contexts/NotificationContext";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import api from "@/lib/axios";

type ForgotPasswordFormData = {
    email: string;
};

type ForgotPasswordResponse = {
    message: string;
};

/**
 * The forgot-password page for starting password recovery.
 * Collects an email address and triggers reset-code delivery.
 *
 * Route: `/auth/forgot-password`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `Form` - Handles forgot-password form submission.
 * - `TextInput` - Captures the account email.
 * - `Button` - Submits the reset request.
 * - `FullscreenCenteredLayout` - Centers page content.
 */
function ForgotPasswordPage() {
    const navigate = useNavigate();
    const { addNotification } = useNotification();
    const { data, errors, setData, setErrors } = useForm<ForgotPasswordFormData>({
        email: "",
    });

    const forgotPasswordMutation = useFormMutation({
        mutationFn: async () => {
            const response = await api.post<ForgotPasswordResponse>(
                "/auth/forgot-password",
                { email: data.email },
            );
            return response.data;
        },
        onSuccess: () => {
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Reset Code Sent",
                message: "Please check your email for the reset code.",
            });
            navigate({
                to: "/auth/verify-reset-code",
                search: { email: data.email },
            });
        },
        onError: setErrors,
        fallbackError: "Failed to send reset code",
    });

    const onSubmit = () => {
        forgotPasswordMutation.mutate();
    };

    return (
        <FullscreenCenteredLayout>
            <section className={styles["forgot-password-page"]}>
                <p className={styles["forgot-password-page__eyebrow"]}>Account recovery</p>
                <h1>Forgot Password</h1>
                <p className={styles["forgot-password-page__lead"]}>
                    Enter your email and we will send a code to get you back in.
                </p>
                <Form onSubmit={onSubmit}>
                    <TextInput
                        name="email"
                        placeholder="Email"
                        data={data}
                        setData={setData}
                        errors={errors}
                    />
                    <Button type="submit">Reset Password</Button>
                </Form>
            </section>
        </FullscreenCenteredLayout>
    );
}

export default ForgotPasswordPage;
