/**
 * Storybook interaction tests for Payments placeholder page behavior.
 *
 * Covered scenarios:
 * - Placeholder content renders for unfinished payments features.
 * - No sidebar controls are rendered at the page-component level.
 */
import { expect, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import PaymentsPage from "@/pages/PaymentsPage/PaymentsPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof PaymentsPage> = {
    title: "Pages/PaymentsPage",
    component: PaymentsPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/payments",
                initialEntries: ["/payments"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof PaymentsPage>;

export const Default: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/payments",
                initialEntries: ["/payments"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Payments" })).toBeVisible();
        await expect(canvas.getByText("Payment tracking is coming soon.")).toBeVisible();
    },
};

export const HidesSidebarControls: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/payments",
                initialEntries: ["/payments"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.queryByRole("button", { name: "Settings" })).toBeNull();
    },
};
