import { createFileRoute } from "@tanstack/react-router";
import PrivateRoute from "@/components/auth/PrivateRoute/PrivateRoute";
import DashboardPage from "@/pages/DashboardPage/DashboardPage";

export const Route = createFileRoute("/dashboard/")({
    component: RouteComponent,
});

export function RouteComponent() {
    return (
        <PrivateRoute>
            <DashboardPage />
        </PrivateRoute>
    );
}
