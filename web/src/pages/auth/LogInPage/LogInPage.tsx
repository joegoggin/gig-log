import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import type { AxiosError } from "axios";
import Button from "@/components/core/Button/Button";
import Checkbox from "@/components/core/CheckBox/CheckBox";
import Form from "@/components/core/Form/Form";
import TextInput from "@/components/core/TextInput/TextInput";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import api from "@/lib/axios";
import type { SetData } from "@/types/SetData";
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
    const [data, setDataState] = useState<LogInFormData>({
        email: "",
        password: "",
        remember_me: false,
    });
    const [errors, setErrors] = useState<Record<string, string>>({});

    const setData: SetData<LogInFormData> = (key, value) => {
        setDataState((prev) => ({ ...prev, [key]: value }));
        setErrors({});
    };

    const loginMutation = useMutation({
        mutationFn: async () => {
            const response = await api.post<LogInResponse>("/auth/log-in", {
                email: data.email,
                password: data.password,
            });
            return response.data;
        },
        onSuccess: () => {
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
