import { createFileRoute } from "@tanstack/react-router";
import JobPage from "@/pages/JobPage/JobPage";

export const Route = createFileRoute("/_authenticated/jobs/$jobId/")({
    component: RouteComponent,
});

export function RouteComponent() {
    const { jobId } = Route.useParams();

    return <JobPage jobId={jobId} />;
}
