import { useNavigate } from "@tanstack/react-router";
import styles from "./LogInPage.module.scss";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import Button from "@/components/core/Button/Button";
import Checkbox from "@/components/core/CheckBox/CheckBox";
import Form from "@/components/core/Form/Form";
import Link from "@/components/core/Link";
import TextInput from "@/components/core/TextInput/TextInput";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import { useAuth } from "@/contexts/AuthContext";
import api from "@/lib/axios";

type LogInFormData = {
    email: string;
    password: string;
    remember_me: boolean;
};

type LogInResponse = {
    message: string;
    user_id: string;
};

/**
 * The authentication page for returning users.
 * Accepts credentials and optional remember-me preference before navigating
 * authenticated users to the dashboard.
 *
 * Route: `/auth/log-in`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `Form` - Handles form submission lifecycle.
 * - `TextInput` - Captures email and password values.
 * - `Checkbox` - Captures remember-me preference.
 * - `Link` - Navigates to password reset flow.
 * - `Button` - Submits the log-in form.
 * - `FullscreenCenteredLayout` - Centers page content.
 */
const LogInPage = () => {
    const navigate = useNavigate();
    const { refreshUser } = useAuth();
    const { data, errors, setData, setErrors } = useForm<LogInFormData>({
        email: "",
        password: "",
        remember_me: false,
    });

    const loginMutation = useFormMutation({
        mutationFn: async () => {
            const response = await api.post<LogInResponse>("/auth/log-in", {
                email: data.email,
                password: data.password,
            });
            return response.data;
        },
        onSuccess: async () => {
            await refreshUser({ throwOnError: true });
            navigate({ to: "/dashboard" });
        },
        onError: setErrors,
        fallbackError: "Login failed",
    });

    const handleSubmit = () => {
        loginMutation.mutate();
    };

    return (
        <FullscreenCenteredLayout className={styles["log-in-page"]}>
            <h1>Log In</h1>
            <Form onSubmit={handleSubmit}>
                <TextInput
                    name="email"
                    placeholder="Email"
                    data={data}
                    setData={setData}
                    errors={errors}
                />
                <TextInput
                    name="password"
                    placeholder="Password"
                    data={data}
                    setData={setData}
                    errors={errors}
                    password
                />
                <Link href="/auth/forgot-password">Forgot Password?</Link>
                <Checkbox
                    name="remember_me"
                    label="Remember me"
                    data={data}
                    setData={setData}
                />
                <Button type="submit">Log In</Button>
            </Form>
        </FullscreenCenteredLayout>
    );
};

export default LogInPage;
