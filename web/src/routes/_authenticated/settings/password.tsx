import { createFileRoute } from "@tanstack/react-router";
import SettingsPasswordPage from "@/pages/SettingsPasswordPage/SettingsPasswordPage";

export const Route = createFileRoute("/_authenticated/settings/password")({
    component: RouteComponent,
});

export function RouteComponent() {
    return <SettingsPasswordPage />;
}
