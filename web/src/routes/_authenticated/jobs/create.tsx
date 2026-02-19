import { createFileRoute } from "@tanstack/react-router";
import CreateJobPage from "@/pages/CreateJobPage/CreateJobPage";

type SearchParams = {
    companyId?: string;
};

export const Route = createFileRoute("/_authenticated/jobs/create")({
    component: RouteComponent,
    validateSearch: (search: Record<string, unknown>): SearchParams => {
        return {
            companyId: typeof search.companyId === "string" ? search.companyId : undefined,
        };
    },
});

export function RouteComponent() {
    const { companyId } = Route.useSearch();

    return <CreateJobPage preselectedCompanyId={companyId} />;
}
