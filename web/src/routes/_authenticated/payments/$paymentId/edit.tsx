import { createFileRoute } from "@tanstack/react-router";
import EditPaymentPage from "@/pages/EditPaymentPage/EditPaymentPage";

export const Route = createFileRoute("/_authenticated/payments/$paymentId/edit")({
    component: RouteComponent,
});

export function RouteComponent() {
    const { paymentId } = Route.useParams();

    return <EditPaymentPage paymentId={paymentId} />;
}
