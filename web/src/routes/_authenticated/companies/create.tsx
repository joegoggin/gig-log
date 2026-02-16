import { createFileRoute } from "@tanstack/react-router";
import CreateCompanyPage from "@/pages/CreateCompanyPage/CreateCompanyPage";

export const Route = createFileRoute("/_authenticated/companies/create")({
    component: RouteComponent,
});

export function RouteComponent() {
    return <CreateCompanyPage />;
}
