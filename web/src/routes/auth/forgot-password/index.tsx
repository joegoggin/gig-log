import { createFileRoute } from "@tanstack/react-router";
import ForgotPasswordPage from "@/pages/auth/ForgotPasswordPage/ForgotPasswordPage";

export const Route = createFileRoute("/auth/forgot-password/")({
    component: RouteComponent,
});

function RouteComponent() {
    return <ForgotPasswordPage />;
}
