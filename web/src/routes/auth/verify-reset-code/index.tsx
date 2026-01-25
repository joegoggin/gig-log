import VerifyResetCodePage from "@/pages/auth/VerifyResetCodePage/VerifyResetCodePage";
import { createFileRoute } from "@tanstack/react-router";

type SearchParams = {
    email?: string;
};

export const Route = createFileRoute("/auth/verify-reset-code/")({
    component: RouteComponent,
    validateSearch: (search: Record<string, unknown>): SearchParams => {
        return {
            email: typeof search.email === "string" ? search.email : undefined,
        };
    },
});

function RouteComponent() {
    const { email } = Route.useSearch();
    return <VerifyResetCodePage email={email} />;
}
