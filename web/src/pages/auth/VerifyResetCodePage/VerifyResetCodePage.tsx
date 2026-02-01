import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import { useNavigate } from "@tanstack/react-router";
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

function VerifyResetCodePage({ email }: VerifyResetCodePageProps) {
    const navigate = useNavigate();
    const { data, errors, setData, setErrors } = useForm<VerifyResetCodeFormData>({
        authCode: "",
    });

    const verifyResetCodeMutation = useFormMutation({
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
        onError: setErrors,
        fallbackError: "Failed to verify reset code",
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
