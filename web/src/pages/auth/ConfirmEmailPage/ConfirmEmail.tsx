import useForm from "@/hooks/useForm";
import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import type { AxiosError } from "axios";
import Button from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import TextInput from "@/components/core/TextInput/TextInput";
import { useNotification } from "@/contexts/NotificationContext";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import api from "@/lib/axios";
import styles from "./ConfirmEmail.module.scss";

type ConfirmEmailProps = {
    email?: string;
};

type ConfirmEmailFormData = {
    authCode: string;
};

type ConfirmEmailResponse = {
    message: string;
};

type ApiErrorResponse = {
    error: string;
};

function ConfirmEmail({ email }: ConfirmEmailProps) {
    const navigate = useNavigate();
    const { addNotification } = useNotification();
    const { data, errors, setData, setErrors } = useForm<ConfirmEmailFormData>({
        authCode: "",
    });

    const confirmEmailMutation = useMutation({
        mutationFn: async () => {
            const response = await api.post<ConfirmEmailResponse>(
                "/auth/confirm-email",
                {
                    email,
                    auth_code: data.authCode,
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
        onError: (error: AxiosError<ApiErrorResponse>) => {
            const message =
                error.response?.data?.error || "Failed to confirm email";
            setErrors({ authCode: message });
        },
    });

    const onSubmit = () => {
        if (!email) {
            setErrors({ authCode: "Email is required" });
            return;
        }
        confirmEmailMutation.mutate();
    };

    return (
        <FullscreenCenteredLayout className={styles["confirm-email"]}>
            <h1>Confirm Email</h1>
            <Form onSubmit={onSubmit}>
                <TextInput
                    name="authCode"
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

export default ConfirmEmail;
