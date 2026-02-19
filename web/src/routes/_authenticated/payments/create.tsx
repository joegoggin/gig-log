import { createFileRoute } from "@tanstack/react-router";
import CreatePaymentPage from "@/pages/CreatePaymentPage/CreatePaymentPage";

type SearchParams = {
    companyId?: string;
};

export const Route = createFileRoute("/_authenticated/payments/create")({
    component: RouteComponent,
    validateSearch: (search: Record<string, unknown>): SearchParams => {
        return {
            companyId: typeof search.companyId === "string" ? search.companyId : undefined,
        };
    },
});

export function RouteComponent() {
    const { companyId } = Route.useSearch();

    return <CreatePaymentPage preselectedCompanyId={companyId} />;
}
