/**
 * Storybook interaction tests for Job page behavior.
 *
 * Covered scenarios:
 * - Hourly jobs render key details and payment-model information.
 * - Payout jobs render payout count and payout amount details.
 * - Missing jobs render the not-found fallback state.
 */
import { expect, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import JobPage from "@/pages/JobPage/JobPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof JobPage> = {
    title: "Pages/JobPage",
    component: JobPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    args: {
        jobId: "123",
    },
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/jobs/123",
                initialEntries: ["/jobs/123"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof JobPage>;

export const ShowsHourlyJobDetails: Story = {
    args: {
        jobId: "123",
        initialJob: {
            id: "123",
            company_id: "c1",
            user_id: "u1",
            title: "Website Maintenance",
            payment_type: "hourly",
            number_of_payouts: null,
            payout_amount: null,
            hourly_rate: "55.50",
            created_at: "2026-01-02T00:00:00Z",
            updated_at: "2026-01-03T00:00:00Z",
        },
    },
    parameters: {
        storyTest: {
            router: {
                storyPath: "/jobs/123",
                initialEntries: ["/jobs/123"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Website Maintenance" })).toBeVisible();
        await expect(canvas.getByText("Payment type")).toBeVisible();
        await expect(canvas.getByText("Hourly")).toBeVisible();
        await expect(canvas.getByText("c1")).toBeVisible();
        await expect(canvas.getByText("Payment model details")).toBeVisible();
        await expect(canvas.getByText("Hourly rate: $55.50/hour")).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Back to Jobs" })).toBeVisible();
    },
};

export const ShowsPayoutJobDetails: Story = {
    args: {
        jobId: "123",
        initialJob: {
            id: "123",
            company_id: "c1",
            user_id: "u1",
            title: "Retainer Package",
            payment_type: "payouts",
            number_of_payouts: 2,
            payout_amount: "350.00",
            hourly_rate: null,
            created_at: "2026-01-02T00:00:00Z",
            updated_at: "2026-01-03T00:00:00Z",
        },
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("Payouts")).toBeVisible();
        await expect(canvas.getByText("2 payouts at $350.00")).toBeVisible();
    },
};

export const ShowsFallbackWhenJobMissing: Story = {
    args: {
        jobId: "123",
        initialJob: null,
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("This job could not be found.")).toBeVisible();
    },
};
