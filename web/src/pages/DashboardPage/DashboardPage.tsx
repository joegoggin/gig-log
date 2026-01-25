import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import type { AxiosError } from "axios";
import Button from "@/components/core/Button/Button";
import { NotificationType } from "@/components/core/Notification/Notification";
import { useAuth } from "@/contexts/AuthContext";
import { useNotification } from "@/contexts/NotificationContext";
import api from "@/lib/axios";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";

type LogOutResponse = {
    message: string;
};

type ApiErrorResponse = {
    error: string;
};


function DashboardPage() {
    const navigate = useNavigate();
    const { setUser } = useAuth();
    const { addNotification } = useNotification();

    const logoutMutation = useMutation({
        mutationFn: async () => {
            const response = await api.post<LogOutResponse>("/auth/log-out");
            return response.data;
        },
        onSuccess: () => {
            setUser(null);
            navigate({ to: "/auth/log-in" });
        },
        onError: (error: AxiosError<ApiErrorResponse>) => {
            const message = error.response?.data?.error || "Failed to log out";
            addNotification({
                type: NotificationType.ERROR,
                title: "Log Out Failed",
                message,
            });
        },
    });

    const handleClick = () => {
        logoutMutation.mutate();
    };

    return (
        <FullscreenCenteredLayout>
            <h1>Dashboard Page</h1>
            <Button type="button" onClick={handleClick}>
                Log Out
            </Button>
        </FullscreenCenteredLayout>
    );
}

export default DashboardPage;
