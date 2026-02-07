import { useEffect } from "react";
import LogInPage from "@/pages/auth/LogInPage/LogInPage";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import Spinner from "@/components/core/Spinner/Spinner";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import { useAuth } from "@/contexts/AuthContext";

export const Route = createFileRoute("/auth/log-in/")({
    component: RouteComponent,
});

export function RouteComponent() {
    const navigate = useNavigate();
    const { isLoggedIn, isLoading } = useAuth();

    useEffect(() => {
        if (!isLoading && isLoggedIn) {
            navigate({ to: "/dashboard" });
        }
    }, [isLoading, isLoggedIn, navigate]);

    if (isLoading) {
        return (
            <FullscreenCenteredLayout>
                <Spinner label="Loading" />
            </FullscreenCenteredLayout>
        );
    }

    if (isLoggedIn) {
        return (
            <FullscreenCenteredLayout>
                <Spinner label="Redirecting to dashboard" />
            </FullscreenCenteredLayout>
        );
    }

    return <LogInPage />;
}
