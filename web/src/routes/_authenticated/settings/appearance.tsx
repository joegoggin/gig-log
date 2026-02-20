import { createFileRoute } from "@tanstack/react-router";
import SettingsAppearancePage from "@/pages/SettingsAppearancePage/SettingsAppearancePage";

export const Route = createFileRoute("/_authenticated/settings/appearance")({
    component: RouteComponent,
});

export function RouteComponent() {
    return <SettingsAppearancePage />;
}
