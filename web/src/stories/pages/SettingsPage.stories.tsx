/**
 * Storybook interaction tests for Settings placeholder page behavior.
 *
 * Covered scenarios:
 * - Placeholder content renders for unfinished settings features.
 * - Sidebar navigation can move from settings to jobs.
 */
import { expect, userEvent, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
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

export const Default: Story = {
    parameters: {
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
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Settings" })).toBeVisible();
        await expect(canvas.getByText("Settings management is coming soon.")).toBeVisible();
    },
};

export const NavigatesToJobs: Story = {
    parameters: {
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
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await userEvent.click(canvas.getByRole("button", { name: "Jobs" }));
        await expect(canvas.getByText("Jobs Route")).toBeVisible();
    },
};
