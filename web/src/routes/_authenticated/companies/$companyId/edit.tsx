import { createFileRoute } from "@tanstack/react-router";
import EditCompanyPage from "@/pages/EditCompanyPage/EditCompanyPage";

export const Route = createFileRoute("/_authenticated/companies/$companyId/edit")({
    component: RouteComponent,
});

export function RouteComponent() {
    const { companyId } = Route.useParams();

    return <EditCompanyPage companyId={companyId} />;
}
