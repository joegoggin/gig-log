import { createFileRoute } from "@tanstack/react-router";
import SettingsPage from "@/pages/SettingsPage/SettingsPage";

export const Route = createFileRoute("/_authenticated/settings/")({
    component: RouteComponent,
});

export function RouteComponent() {
    return <SettingsPage />;
}
