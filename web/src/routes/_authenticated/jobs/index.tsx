import { createFileRoute } from "@tanstack/react-router";
import JobsPage from "@/pages/JobsPage/JobsPage";

export const Route = createFileRoute("/_authenticated/jobs/")({
    component: RouteComponent,
});

export function RouteComponent() {
    return <JobsPage />;
}
