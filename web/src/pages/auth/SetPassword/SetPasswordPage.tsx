import { useNavigate } from "@tanstack/react-router";
import styles from "./SetPasswordPage.module.scss";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import Button from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import TextInput from "@/components/core/TextInput/TextInput";
import { useNotification } from "@/contexts/NotificationContext";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import api from "@/lib/axios";

type SetPasswordFormData = {
    password: string;
    confirm: string;
};

type SetPasswordResponse = {
    message: string;
};

/**
 * The password reset completion page.
 * Allows users to set and confirm a new password after verification.
 *
 * Route: `/auth/set-password`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `Form` - Handles password reset submission.
 * - `TextInput` - Captures password and confirmation fields.
 * - `Button` - Submits the new password.
 * - `FullscreenCenteredLayout` - Centers page content.
 */
function SetPasswordPage() {
    const navigate = useNavigate();
    const { addNotification } = useNotification();
    const { data, errors, setData, setErrors } = useForm<SetPasswordFormData>({
        password: "",
        confirm: "",
    });

    const setPasswordMutation = useFormMutation({
        mutationFn: async () => {
            const response = await api.post<SetPasswordResponse>(
                "/auth/set-password",
                {
                    password: data.password,
                    confirm: data.confirm,
                },
            );
            return response.data;
        },
        onSuccess: () => {
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Password Reset",
                message: "Your password has been reset successfully.",
            });
            navigate({ to: "/auth/log-in" });
        },
        onError: setErrors,
        fallbackError: "Failed to reset password",
    });

    const onSubmit = () => {
        setPasswordMutation.mutate();
    };

    return (
        <FullscreenCenteredLayout className={styles["set-password-page"]}>
            <h1>Set Password</h1>
            <Form onSubmit={onSubmit}>
                <TextInput
                    name="password"
                    placeholder="Password"
                    data={data}
                    setData={setData}
                    errors={errors}
                    password
                />
                <TextInput
                    name="confirm"
                    placeholder="Confirm Password"
                    data={data}
                    setData={setData}
                    errors={errors}
                    password
                />
                <Button type="submit">Set Password</Button>
            </Form>
        </FullscreenCenteredLayout>
    );
}

export default SetPasswordPage;
