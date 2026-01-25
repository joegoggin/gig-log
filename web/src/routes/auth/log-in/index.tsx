import LogInPage from "@/pages/auth/LogInPage/LogInPage";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/auth/log-in/")({
    component: LogInPage,
});
