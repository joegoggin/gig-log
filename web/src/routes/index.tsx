import { createFileRoute } from "@tanstack/react-router";
import { useAuth } from "@/contexts/AuthContext";
import HomePage from "@/pages/HomePage/HomePage";

export const Route = createFileRoute("/")({
    component: RouteComponent,
});

function RouteComponent() {
    const { isLoggedIn } = useAuth();

    return <HomePage isLoggedIn={isLoggedIn} />;
}
