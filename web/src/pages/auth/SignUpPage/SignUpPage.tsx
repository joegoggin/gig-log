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
import styles from "./SignUpPage.module.scss";

type SignUpFormData = {
    first_name: string;
    last_name: string;
    email: string;
    password: string;
    confirm: string;
};

type SignUpResponse = {
    message: string;
};

type ApiErrorResponse = {
    error: string;
};

const SignUpPage: React.FC = () => {
    const navigate = useNavigate();
    const { addNotification } = useNotification();
    const { data, errors, setData, setErrors } = useForm<SignUpFormData>({
        first_name: "",
        last_name: "",
        email: "",
        password: "",
        confirm: "",
    });

    const signUpMutation = useMutation({
        mutationFn: async () => {
            const response = await api.post<SignUpResponse>("/auth/sign-up", {
                first_name: data.first_name,
                last_name: data.last_name,
                email: data.email,
                password: data.password,
                confirm: data.confirm,
            });
            return response.data;
        },
        onSuccess: () => {
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Account Created",
                message: "Please check your email to confirm your account.",
            });
            navigate({
                to: "/auth/confirm-email",
                search: { email: data.email },
            });
        },
        onError: (error: AxiosError<ApiErrorResponse>) => {
            const message = error.response?.data?.error || "Sign up failed";
            setErrors({ email: message });
        },
    });

    const handleSubmit = () => {
        if (data.password !== data.confirm) {
            setErrors({ confirm: "Passwords do not match" });
            return;
        }
        signUpMutation.mutate();
    };

    return (
        <FullscreenCenteredLayout className={styles["sign-up"]}>
            <h1>Sign Up</h1>
            <Form onSubmit={handleSubmit}>
                <TextInput
                    name="first_name"
                    placeholder="First Name"
                    data={data}
                    setData={setData}
                    errors={errors}
                />
                <TextInput
                    name="last_name"
                    placeholder="Last Name"
                    data={data}
                    setData={setData}
                    errors={errors}
                />
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
                <TextInput
                    name="confirm"
                    placeholder="Confirm Password"
                    data={data}
                    setData={setData}
                    errors={errors}
                    password
                />
                <Button type="submit">Sign Up</Button>
            </Form>
        </FullscreenCenteredLayout>
    );
};

export default SignUpPage;
