import { createFileRoute } from "@tanstack/react-router";
import PaymentsPage from "@/pages/PaymentsPage/PaymentsPage";

export const Route = createFileRoute("/_authenticated/payments/")({
    component: RouteComponent,
});

export function RouteComponent() {
    return <PaymentsPage />;
}
