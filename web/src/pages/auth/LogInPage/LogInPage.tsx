import useForm from "@/hooks/useForm";
import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import type { AxiosError } from "axios";
import Button from "@/components/core/Button/Button";
import Checkbox from "@/components/core/CheckBox/CheckBox";
import Form from "@/components/core/Form/Form";
import Link from "@/components/core/Link";
import TextInput from "@/components/core/TextInput/TextInput";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import { useAuth } from "@/contexts/AuthContext";
import api from "@/lib/axios";
import styles from "./LogInPage.module.scss";

type LogInFormData = {
    email: string;
    password: string;
    remember_me: boolean;
};

type LogInResponse = {
    message: string;
    user_id: string;
};

type ApiErrorResponse = {
    error: string;
};

const LogInPage = () => {
    const navigate = useNavigate();
    const { refreshUser } = useAuth();
    const { data, errors, setData, setErrors } = useForm<LogInFormData>({
        email: "",
        password: "",
        remember_me: false,
    });

    const loginMutation = useMutation({
        mutationFn: async () => {
            const response = await api.post<LogInResponse>("/auth/log-in", {
                email: data.email,
                password: data.password,
            });
            return response.data;
        },
        onSuccess: async () => {
            await refreshUser();
            navigate({ to: "/dashboard" });
        },
        onError: (error: AxiosError<ApiErrorResponse>) => {
            const message = error.response?.data?.error || "Login failed";
            setErrors({ email: message });
        },
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
