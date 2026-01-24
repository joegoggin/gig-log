import { createFileRoute } from "@tanstack/react-router";
import HomePage from "@/pages/HomePage/HomePage";

export const Route = createFileRoute("/")({
    component: RouteComponent,
});

function RouteComponent() {
    // TODO: Replace with actual auth state from auth context/store
    const isLoggedIn = false;

    return <HomePage isLoggedIn={isLoggedIn} />;
}
