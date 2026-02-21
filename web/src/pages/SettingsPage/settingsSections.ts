export type SettingsSection = {
    /** Route path for the settings section */
    href: string;
    /** Short label used in navigation */
    label: string;
    /** Supporting description used in the settings hub */
    description: string;
};

export const settingsSections: Array<SettingsSection> = [
    {
        href: "/settings",
        label: "Overview",
        description:
            "Start here to choose the settings area you want to manage.",
    },
    {
        href: "/settings/password",
        label: "Password",
        description: "Update your password using your current credentials.",
    },
    {
        href: "/settings/email",
        label: "Email",
        description: "Change your login email with a confirmation-code flow.",
    },
    {
        href: "/settings/appearance",
        label: "Appearance",
        description: "Adjust theme mode, presets, and custom color palettes.",
    },
];
