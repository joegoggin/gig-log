import { createFileRoute } from "@tanstack/react-router";
import CompaniesPage from "@/pages/CompaniesPage/CompaniesPage";

export const Route = createFileRoute("/_authenticated/companies/")({
    component: RouteComponent,
});

export function RouteComponent() {
    return <CompaniesPage />;
}
