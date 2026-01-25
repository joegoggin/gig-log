import useForm from "@/hooks/useForm";
import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import type { AxiosError } from "axios";
import Button from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import TextInput from "@/components/core/TextInput/TextInput";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import api from "@/lib/axios";
import styles from "./VerifyResetCodePage.module.scss";

type VerifyResetCodePageProps = {
    email?: string;
};

type VerifyResetCodeFormData = {
    authCode: string;
};

type VerifyResetCodeResponse = {
    message: string;
};

type ApiErrorResponse = {
    error: string;
};

function VerifyResetCodePage({ email }: VerifyResetCodePageProps) {
    const navigate = useNavigate();
    const { data, errors, setData, setErrors } = useForm<VerifyResetCodeFormData>({
        authCode: "",
    });

    const verifyResetCodeMutation = useMutation({
        mutationFn: async () => {
            const response = await api.post<VerifyResetCodeResponse>(
                "/auth/verify-forgot-password",
                {
                    email,
                    auth_code: data.authCode,
                },
            );
            return response.data;
        },
        onSuccess: () => {
            navigate({ to: "/auth/set-password" });
        },
        onError: (error: AxiosError<ApiErrorResponse>) => {
            const message =
                error.response?.data?.error || "Failed to verify reset code";
            setErrors({ authCode: message });
        },
    });

    const onSubmit = () => {
        if (!email) {
            setErrors({ authCode: "Email is required" });
            return;
        }
        verifyResetCodeMutation.mutate();
    };

    return (
        <FullscreenCenteredLayout className={styles["verify-reset-code-page"]}>
            <h1>Verify Reset Code</h1>
            <Form onSubmit={onSubmit}>
                <TextInput
                    name="authCode"
                    placeholder="Enter reset code"
                    data={data}
                    setData={setData}
                    errors={errors}
                />
                <Button type="submit">Verify Code</Button>
            </Form>
        </FullscreenCenteredLayout>
    );
}

export default VerifyResetCodePage;
