import { Outlet, createFileRoute } from "@tanstack/react-router";
import PrivateRoute from "@/components/auth/PrivateRoute/PrivateRoute";
import MainLayout from "@/layouts/MainLayout/MainLayout";
import { WorkSessionProvider } from "@/contexts/WorkSessionContext";

export const Route = createFileRoute("/_authenticated")({
    component: RouteComponent,
});

export function RouteComponent() {
    return (
        <PrivateRoute>
            <WorkSessionProvider>
                <MainLayout>
                    <Outlet />
                </MainLayout>
            </WorkSessionProvider>
        </PrivateRoute>
    );
}
