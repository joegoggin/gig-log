import { createFileRoute } from "@tanstack/react-router";
import CompanyPage from "@/pages/CompanyPage/CompanyPage";

export const Route = createFileRoute("/_authenticated/companies/$companyId/")({
    component: RouteComponent,
});

export function RouteComponent() {
    const { companyId } = Route.useParams();

    return <CompanyPage companyId={companyId} />;
}
