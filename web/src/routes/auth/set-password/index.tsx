import { createFileRoute } from "@tanstack/react-router";
import PrivateRoute from "@/components/auth/PrivateRoute/PrivateRoute";
import SetPasswordPage from "@/pages/auth/SetPassword/SetPasswordPage";

export const Route = createFileRoute("/auth/set-password/")({
    component: RouteComponent,
});

export function RouteComponent() {
    return (
        <PrivateRoute>
            <SetPasswordPage />
        </PrivateRoute>
    );
}
