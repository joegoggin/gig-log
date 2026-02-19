import { createFileRoute } from "@tanstack/react-router";
import PaymentPage from "@/pages/PaymentPage/PaymentPage";

export const Route = createFileRoute("/_authenticated/payments/$paymentId/")({
    component: RouteComponent,
});

export function RouteComponent() {
    const { paymentId } = Route.useParams();

    return <PaymentPage paymentId={paymentId} />;
}
