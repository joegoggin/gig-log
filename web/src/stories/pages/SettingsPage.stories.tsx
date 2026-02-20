/**
 * Storybook interaction tests for the settings hub page.
 *
 * Covered scenarios:
 * - The settings hub renders dedicated cards for password, email, and appearance.
 * - Section actions navigate to the expected settings sub-routes.
 *
 * These tests prevent regressions where split settings navigation becomes
 * disconnected from route structure.
 */
import { expect, userEvent, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import SettingsPage from "@/pages/SettingsPage/SettingsPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof SettingsPage> = {
    title: "Pages/SettingsPage",
    component: SettingsPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/settings",
                initialEntries: ["/settings"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof SettingsPage>;

export const RendersDedicatedSettingsSections: Story = {
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);

        await expect(canvas.getByRole("heading", { name: "Settings" })).toBeVisible();
        await expect(canvas.getByRole("heading", { name: "Password" })).toBeVisible();
        await expect(canvas.getByRole("heading", { name: "Email" })).toBeVisible();
        await expect(canvas.getByRole("heading", { name: "Appearance" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Open Password" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Open Email" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Open Appearance" })).toBeVisible();
    },
};

export const RoutesToSettingsSubPages: Story = {
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);

        await userEvent.click(canvas.getByRole("button", { name: "Open Password" }));
        await expect(canvas.getByText("Settings Password Route")).toBeVisible();
    },
};
