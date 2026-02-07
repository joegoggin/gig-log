import ConfirmEmailPage from "@/pages/auth/ConfirmEmailPage/ConfirmEmail";
import { createFileRoute } from "@tanstack/react-router";

type SearchParams = {
    email?: string;
};

export const Route = createFileRoute("/auth/confirm-email/")({
    component: RouteComponent,
    validateSearch: (search: Record<string, unknown>): SearchParams => {
        return {
            email: typeof search.email === "string" ? search.email : undefined,
        };
    },
});

function RouteComponent() {
    const { email } = Route.useSearch();
    return <ConfirmEmailPage email={email} />;
}
