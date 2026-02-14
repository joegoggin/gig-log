/**
 * Storybook interaction tests for Companies placeholder page behavior.
 *
 * Covered scenarios:
 * - Placeholder content renders for unfinished companies features.
 * - No sidebar controls are rendered at the page-component level.
 */
import { expect, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import CompaniesPage from "@/pages/CompaniesPage/CompaniesPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof CompaniesPage> = {
    title: "Pages/CompaniesPage",
    component: CompaniesPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/companies",
                initialEntries: ["/companies"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof CompaniesPage>;

export const Default: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/companies",
                initialEntries: ["/companies"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Companies" })).toBeVisible();
        await expect(canvas.getByText("Company management is coming soon.")).toBeVisible();
    },
};

export const HidesSidebarControls: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/companies",
                initialEntries: ["/companies"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.queryByRole("button", { name: "Dashboard" })).toBeNull();
    },
};
