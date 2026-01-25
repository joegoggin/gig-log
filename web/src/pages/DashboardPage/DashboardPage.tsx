import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import Button from "@/components/core/Button/Button";
import api from "@/lib/axios";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";

type LogOutResponse = {
    message: string;
};

function DashboardPage() {
    const navigate = useNavigate();

    const logoutMutation = useMutation({
        mutationFn: async () => {
            const response = await api.post<LogOutResponse>("/auth/log-out");
            return response.data;
        },
        onSuccess: () => {
            navigate({ to: "/auth/log-in" });
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
