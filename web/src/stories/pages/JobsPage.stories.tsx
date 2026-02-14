/**
 * Storybook interaction tests for Jobs placeholder page behavior.
 *
 * Covered scenarios:
 * - Placeholder content renders for unfinished jobs features.
 * - Sidebar navigation can move from jobs to payments.
 */
import { expect, userEvent, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import JobsPage from "@/pages/JobsPage/JobsPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof JobsPage> = {
    title: "Pages/JobsPage",
    component: JobsPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/jobs",
                initialEntries: ["/jobs"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof JobsPage>;

export const Default: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/jobs",
                initialEntries: ["/jobs"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Jobs" })).toBeVisible();
        await expect(canvas.getByText("Job tracking is coming soon.")).toBeVisible();
    },
};

export const NavigatesToPayments: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/jobs",
                initialEntries: ["/jobs"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await userEvent.click(canvas.getByRole("button", { name: "Payments" }));
        await expect(canvas.getByText("Payments Route")).toBeVisible();
    },
};
