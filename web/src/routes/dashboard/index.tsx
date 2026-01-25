import PrivateRoute from "@/components/auth/PrivateRoute/PrivateRoute";
import DashboardPage from "@/pages/DashboardPage/DashboardPage";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/dashboard/")({
    component: RouteComponent,
});

function RouteComponent() {
    return (
        <PrivateRoute>
            <DashboardPage />
        </PrivateRoute>
    );
}
