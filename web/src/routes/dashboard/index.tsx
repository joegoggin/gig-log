import DashboardPage from "@/pages/DashboardPage/DashboardPage";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/dashboard/")({
    component: RouteComponent,
});

function RouteComponent() {
    return <DashboardPage />;
}
