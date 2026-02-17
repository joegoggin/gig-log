/**
 * Storybook interaction tests for Dashboard page behavior.
 *
 * Covered scenarios:
 * - Dashboard content renders inside the authenticated main layout.
 * - No sidebar controls are rendered at the page-component level.
 */
import { expect, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import DashboardPage from "@/pages/DashboardPage/DashboardPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof DashboardPage> = {
    title: "Pages/DashboardPage",
    component: DashboardPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/dashboard",
                initialEntries: ["/dashboard"],
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof DashboardPage>;

export const Default: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/dashboard",
                initialEntries: ["/dashboard"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Dashboard" })).toBeVisible();
        await expect(
            canvas.getByText("Welcome back. Use the navigation menu to move across the app."),
        ).toBeVisible();
    },
};

export const HidesSidebarControls: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/dashboard",
                initialEntries: ["/dashboard"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.queryByRole("button", { name: "Companies" })).toBeNull();
    },
};
