import { useNavigate } from "@tanstack/react-router";
import styles from "./VerifyResetCodePage.module.scss";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import Button from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import TextInput from "@/components/core/TextInput/TextInput";
import { useAuth } from "@/contexts/AuthContext";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import api from "@/lib/axios";

type VerifyResetCodePageProps = {
    /** Email tied to the reset-code request */
    email?: string;
};

type VerifyResetCodeFormData = {
    auth_code: string;
};

type VerifyResetCodeResponse = {
    message: string;
};

/**
 * The reset-code verification page in the password recovery flow.
 * Verifies a one-time code sent to the user's email before allowing
 * password updates.
 *
 * Route: `/auth/verify-reset-code`
 *
 * ## Props
 *
 * - `email` - Email address associated with the password reset request
 *
 * ## Related Components
 *
 * - `Form` - Handles verification code submission.
 * - `TextInput` - Captures the reset code.
 * - `Button` - Submits code verification.
 * - `FullscreenCenteredLayout` - Centers page content.
 */
function VerifyResetCodePage({ email }: VerifyResetCodePageProps) {
    const navigate = useNavigate();
    const { refreshUser } = useAuth();
    const { data, errors, setData, setErrors } = useForm<VerifyResetCodeFormData>({
        auth_code: "",
    });

    const verifyResetCodeMutation = useFormMutation({
        mutationFn: async () => {
            const response = await api.post<VerifyResetCodeResponse>(
                "/auth/verify-forgot-password",
                {
                    email,
                    auth_code: data.auth_code,
                },
            );
            return response.data;
        },
        onSuccess: async () => {
            await refreshUser();
            navigate({ to: "/auth/set-password" });
        },
        onError: setErrors,
        fallbackError: "Failed to verify reset code",
    });

    const onSubmit = () => {
        if (!email) {
            setErrors({ auth_code: "Email is required" });
            return;
        }
        verifyResetCodeMutation.mutate();
    };

    return (
        <FullscreenCenteredLayout className={styles["verify-reset-code-page"]}>
            <h1>Verify Reset Code</h1>
            <Form onSubmit={onSubmit}>
                <TextInput
                    name="auth_code"
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
