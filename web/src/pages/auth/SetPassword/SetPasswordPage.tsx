import { useState } from "react";
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
import type { SetData } from "@/types/SetData";
import styles from "./SetPasswordPage.module.scss";

type SetPasswordFormData = {
    password: string;
    confirm: string;
};

type SetPasswordResponse = {
    message: string;
};

type ApiErrorResponse = {
    error: string;
};

function SetPasswordPage() {
    const navigate = useNavigate();
    const { addNotification } = useNotification();
    const [data, setDataState] = useState<SetPasswordFormData>({
        password: "",
        confirm: "",
    });
    const [errors, setErrors] = useState<Record<string, string>>({});

    const setData: SetData<SetPasswordFormData> = (key, value) => {
        setDataState((prev) => ({ ...prev, [key]: value }));
        setErrors({});
    };

    const setPasswordMutation = useMutation({
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
        onError: (error: AxiosError<ApiErrorResponse>) => {
            const message =
                error.response?.data?.error || "Failed to reset password";
            setErrors({ password: message });
        },
    });

    const onSubmit = () => {
        if (data.password !== data.confirm) {
            setErrors({ confirm: "Passwords do not match" });
            return;
        }
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
