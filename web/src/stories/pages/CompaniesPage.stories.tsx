/**
 * Storybook interaction tests for Companies page behavior.
 *
 * Covered scenarios:
 * - Company cards expose CRUD-adjacent actions used in day-to-day workflows.
 * - Navigation actions route to job creation, payments, and company detail/edit paths.
 * - Delete requests remove cards on success and surface notifications on failure.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import CompaniesPage from "@/pages/CompaniesPage/CompaniesPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import {
    createAxiosErrorResponse,
    createMockApiResponse,
    mockApiDeleteHandler,
} from "@/test-utils/mockApiClient";

const addNotificationSpy = fn();

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

const companiesFixture = [
    {
        id: "123",
        user_id: "u1",
        name: "Acme Studio",
        requires_tax_withholdings: true,
        tax_withholding_rate: "30.00",
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
    },
];

export const RendersCompanyCardsAndActions: Story = {
    args: {
        initialCompanies: companiesFixture,
    },
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
        await expect(canvas.getByText("Acme Studio")).toBeVisible();
        await expect(canvas.getByText("Tax withholdings: Enabled (30.00%)")).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Add Job" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Add Payment" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "View Company" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Edit Company" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Delete Company" })).toBeVisible();
    },
};

export const RoutesAddJobToCreateJobPage: Story = {
    args: {
        initialCompanies: companiesFixture,
    },
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
        await userEvent.click(canvas.getByRole("button", { name: "Add Job" }));
        await expect(canvas.getByText("Create Job Route")).toBeVisible();
    },
};

export const RoutesAddPaymentToCreatePaymentPage: Story = {
    args: {
        initialCompanies: companiesFixture,
    },
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
        await userEvent.click(canvas.getByRole("button", { name: "Add Payment" }));
        await expect(canvas.getByText("Create Payment Route")).toBeVisible();
    },
};

export const RoutesViewCompanyToDetailRoute: Story = {
    args: {
        initialCompanies: companiesFixture,
    },
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
        await userEvent.click(canvas.getByRole("button", { name: "View Company" }));
        await expect(canvas.getByText("Company Route")).toBeVisible();
    },
};

export const DeletesCompanyAndRemovesCard: Story = {
    args: {
        initialCompanies: companiesFixture,
    },
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
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const restoreDelete = mockApiDeleteHandler(async (url) => {
            await expect(url).toBe("/companies/123");
            return createMockApiResponse({ message: "Company deleted successfully." });
        });
        const originalConfirm = window.confirm;
        window.confirm = () => true;
        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await expect(canvas.getByText("Acme Studio")).toBeVisible();

            await userEvent.click(canvas.getByRole("button", { name: "Delete Company" }));

            await waitFor(() => {
                expect(canvas.queryByText("Acme Studio")).toBeNull();
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Company Deleted",
                message: "Acme Studio was deleted successfully.",
            });
        } finally {
            restoreDelete();
            window.confirm = originalConfirm;
        }
    },
};

export const ShowsErrorWhenDeleteFails: Story = {
    args: {
        initialCompanies: companiesFixture,
    },
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
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const restoreDelete = mockApiDeleteHandler(() =>
            Promise.reject(
                createAxiosErrorResponse({ message: "Delete failed" }, 500, "Server Error"),
            ),
        );
        const originalConfirm = window.confirm;
        window.confirm = () => true;
        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.click(canvas.getByRole("button", { name: "Delete Company" }));

            await expect(canvas.getByText("Acme Studio")).toBeVisible();
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.ERROR,
                title: "Delete Failed",
                message: "Unable to delete this company right now.",
            });
        } finally {
            restoreDelete();
            window.confirm = originalConfirm;
        }
    },
};
