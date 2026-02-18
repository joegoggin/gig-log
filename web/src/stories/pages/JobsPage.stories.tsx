/**
 * Storybook interaction tests for Jobs page behavior.
 *
 * Covered scenarios:
 * - Job cards render with action controls and payment details.
 * - Delete success removes the card and dispatches a success notification.
 * - Delete failures preserve the card and dispatch an error notification.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import JobsPage from "@/pages/JobsPage/JobsPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import {
    createAxiosErrorResponse,
    createMockApiResponse,
    mockApiDeleteHandler,
} from "@/test-utils/mockApiClient";

const addNotificationSpy = fn();

const jobsFixture = [
    {
        id: "j1",
        company_id: "c1",
        user_id: "u1",
        title: "Website Maintenance",
        payment_type: "hourly" as const,
        number_of_payouts: null,
        payout_amount: null,
        hourly_rate: "55.50",
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-02T00:00:00Z",
    },
];

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

export const RendersJobCardsAndActions: Story = {
    args: {
        initialJobs: jobsFixture,
    },
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
        await expect(canvas.getByText("Website Maintenance")).toBeVisible();
        await expect(canvas.getByText("Payment type: Hourly")).toBeVisible();
        await expect(canvas.getByText("Rate: $55.50/hour")).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Create Job (coming soon)" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "View Job" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Edit Job" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Delete Job" })).toBeVisible();
    },
};

export const DeletesJobAndRemovesCard: Story = {
    args: {
        initialJobs: jobsFixture,
    },
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
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const restoreDelete = mockApiDeleteHandler(async (url) => {
            await expect(url).toBe("/jobs/j1");
            return createMockApiResponse({ message: "Job deleted successfully." });
        });
        const originalConfirm = window.confirm;
        window.confirm = () => true;
        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await expect(canvas.getByText("Website Maintenance")).toBeVisible();

            await userEvent.click(canvas.getByRole("button", { name: "Delete Job" }));

            await waitFor(() => {
                expect(canvas.queryByText("Website Maintenance")).toBeNull();
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Job Deleted",
                message: "Website Maintenance was deleted successfully.",
            });
        } finally {
            restoreDelete();
            window.confirm = originalConfirm;
        }
    },
};

export const ShowsErrorWhenDeleteFails: Story = {
    args: {
        initialJobs: jobsFixture,
    },
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
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const restoreDelete = mockApiDeleteHandler(() =>
            Promise.reject(createAxiosErrorResponse({ message: "Delete failed" }, 500, "Error")),
        );
        const originalConfirm = window.confirm;
        window.confirm = () => true;
        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.click(canvas.getByRole("button", { name: "Delete Job" }));

            await expect(canvas.getByText("Website Maintenance")).toBeVisible();
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.ERROR,
                title: "Delete Failed",
                message: "Unable to delete this job right now.",
            });
        } finally {
            restoreDelete();
            window.confirm = originalConfirm;
        }
    },
};

export const ShowsEmptyState: Story = {
    args: {
        initialJobs: [],
    },
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
        await expect(
            canvas.getByText("No jobs yet. Create your first job to start tracking work."),
        ).toBeVisible();
    },
};
