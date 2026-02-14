import { Outlet, createFileRoute } from "@tanstack/react-router";
import PrivateRoute from "@/components/auth/PrivateRoute/PrivateRoute";
import MainLayout from "@/layouts/MainLayout/MainLayout";

export const Route = createFileRoute("/_authenticated")({
    component: RouteComponent,
});

function RouteComponent() {
    return (
        <PrivateRoute>
            <MainLayout>
                <Outlet />
            </MainLayout>
        </PrivateRoute>
    );
}
