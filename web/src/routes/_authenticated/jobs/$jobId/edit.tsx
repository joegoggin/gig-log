import { createFileRoute } from "@tanstack/react-router";
import EditJobPage from "@/pages/EditJobPage/EditJobPage";

export const Route = createFileRoute("/_authenticated/jobs/$jobId/edit")({
    component: RouteComponent,
});

export function RouteComponent() {
    const { jobId } = Route.useParams();

    return <EditJobPage jobId={jobId} />;
}
