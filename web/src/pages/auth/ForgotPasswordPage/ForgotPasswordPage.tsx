import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import { useNavigate } from "@tanstack/react-router";
import Button from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import TextInput from "@/components/core/TextInput/TextInput";
import { useNotification } from "@/contexts/NotificationContext";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import api from "@/lib/axios";
import styles from "./ForgotPasswordPage.module.scss";

type ForgotPasswordFormData = {
    email: string;
};

type ForgotPasswordResponse = {
    message: string;
};

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
        <FullscreenCenteredLayout className={styles["forgot-password-page"]}>
            <h1>Forgot Password</h1>
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
        </FullscreenCenteredLayout>
    );
}

export default ForgotPasswordPage;
