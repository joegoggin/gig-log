import { createFileRoute } from "@tanstack/react-router";
import SettingsEmailPage from "@/pages/SettingsEmailPage/SettingsEmailPage";

export const Route = createFileRoute("/_authenticated/settings/email")({
    component: RouteComponent,
});

export function RouteComponent() {
    return <SettingsEmailPage />;
}
