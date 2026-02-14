import { Navigate, createFileRoute } from "@tanstack/react-router";
import LogInPage from "@/pages/auth/LogInPage/LogInPage";
import Spinner from "@/components/core/Spinner/Spinner";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import { useAuth } from "@/contexts/AuthContext";

export const Route = createFileRoute("/auth/log-in/")({
    component: RouteComponent,
});

export function RouteComponent() {
    const { isLoggedIn, isLoading } = useAuth();

    if (isLoading) {
        return (
            <FullscreenCenteredLayout>
                <Spinner label="Loading" />
            </FullscreenCenteredLayout>
        );
    }

    if (isLoggedIn) {
        return <Navigate to="/dashboard" />;
    }

    return <LogInPage />;
}
