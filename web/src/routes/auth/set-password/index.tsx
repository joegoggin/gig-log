import PrivateRoute from "@/components/auth/PrivateRoute/PrivateRoute";
import SetPasswordPage from "@/pages/auth/SetPassword/SetPasswordPage";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/auth/set-password/")({
    component: RouteComponent,
});

function RouteComponent() {
    return (
        <PrivateRoute>
            <SetPasswordPage />
        </PrivateRoute>
    );
}
