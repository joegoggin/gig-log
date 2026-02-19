/**
 * Storybook interaction tests for Payments page behavior.
 *
 * Covered scenarios:
 * - Payment cards render with status details and create/view/edit/delete actions.
 * - Navigation actions route to create, detail, and edit payment paths.
 * - Delete requests remove cards on success and surface notifications on failure.
 * - Initial load failures render retry UI and recover on retry.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import type { Payment } from "@/types/models/Payment";
import { NotificationType } from "@/components/core/Notification/Notification";
import PaymentsPage from "@/pages/PaymentsPage/PaymentsPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import {
    createAxiosErrorResponse,
    createMockApiResponse,
    mockApiDeleteHandler,
    mockApiGetHandler,
} from "@/test-utils/mockApiClient";

const addNotificationSpy = fn();

const paymentsFixture: Array<Payment> = [
    {
        id: "p1",
        user_id: "u1",
        company_id: "c1",
        total: "500.00",
        payout_type: "direct_deposit",
        expected_payout_date: "2026-03-10",
        expected_transfer_date: "2026-03-12",
        transfer_initiated: true,
        payment_received: true,
        transfer_received: true,
        tax_withholdings_covered: false,
        created_at: "2026-03-01T00:00:00Z",
        updated_at: "2026-03-02T00:00:00Z",
    },
];

type StoryLoadedContext = {
    restoreGet: () => void;
};

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

export const RendersPaymentCardsAndActions: Story = {
    args: {
        initialPayments: paymentsFixture,
    },
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
        await expect(canvas.getByText("Total: $500.00")).toBeVisible();
        await expect(canvas.getByText("Payout type: Direct Deposit")).toBeVisible();
        await expect(canvas.getByText("Payment received: Yes")).toBeVisible();
        await expect(canvas.getByText("Transfer received: Yes")).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Create Payment" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "View Payment" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Edit Payment" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Delete Payment" })).toBeVisible();
    },
};

export const RoutesCreatePaymentToCreatePage: Story = {
    args: {
        initialPayments: paymentsFixture,
    },
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
        await userEvent.click(canvas.getByRole("button", { name: "Create Payment" }));
        await expect(canvas.getByText("Create Payment Route")).toBeVisible();
    },
};

export const RoutesViewPaymentToDetailPage: Story = {
    args: {
        initialPayments: paymentsFixture,
    },
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
        await userEvent.click(canvas.getByRole("button", { name: "View Payment" }));
        await expect(canvas.getByText("Payment Route")).toBeVisible();
    },
};

export const RoutesEditPaymentToEditPage: Story = {
    args: {
        initialPayments: paymentsFixture,
    },
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
        await userEvent.click(canvas.getByRole("button", { name: "Edit Payment" }));
        await expect(canvas.getByText("Edit Payment Route")).toBeVisible();
    },
};

export const DeletesPaymentAndRemovesCard: Story = {
    args: {
        initialPayments: paymentsFixture,
    },
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
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const restoreDelete = mockApiDeleteHandler(async (url) => {
            await expect(url).toBe("/payments/p1");
            return createMockApiResponse({ message: "Payment deleted successfully." });
        });
        const originalConfirm = window.confirm;
        window.confirm = () => true;
        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await expect(canvas.getByText("Total: $500.00")).toBeVisible();

            await userEvent.click(canvas.getByRole("button", { name: "Delete Payment" }));

            await waitFor(() => {
                expect(canvas.queryByText("Total: $500.00")).toBeNull();
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Payment Deleted",
                message: "Payment was deleted successfully.",
            });
        } finally {
            restoreDelete();
            window.confirm = originalConfirm;
        }
    },
};

export const ShowsErrorWhenDeleteFails: Story = {
    args: {
        initialPayments: paymentsFixture,
    },
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
            await userEvent.click(canvas.getByRole("button", { name: "Delete Payment" }));

            await expect(canvas.getByText("Total: $500.00")).toBeVisible();
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.ERROR,
                title: "Delete Failed",
                message: "Unable to delete this payment right now.",
            });
        } finally {
            restoreDelete();
            window.confirm = originalConfirm;
        }
    },
};

export const RetriesAfterInitialLoadFailure: Story = {
    loaders: [
        () => {
            addNotificationSpy.mockClear();
            let requestCount = 0;

            return {
                restoreGet: mockApiGetHandler((url) => {
                    if (url === "/payments") {
                        requestCount += 1;

                        if (requestCount === 1) {
                            return Promise.reject(
                                createAxiosErrorResponse({ message: "Unable to load payments" }),
                            );
                        }

                        return Promise.resolve(createMockApiResponse({ payments: paymentsFixture }));
                    }

                    return Promise.resolve(createMockApiResponse({}));
                }),
            };
        },
    ],
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
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement, loaded }) => {
        const { restoreGet } = loaded as StoryLoadedContext;

        try {
            const canvas = within(canvasElement);

            await waitFor(() => {
                expect(canvas.getByText("Unable to load payments right now.")).toBeVisible();
            });

            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.ERROR,
                title: "Payments Unavailable",
                message: "Failed to load payments.",
            });

            await userEvent.click(canvas.getByRole("button", { name: "Retry" }));

            await waitFor(() => {
                expect(canvas.getByText("Total: $500.00")).toBeVisible();
            });
        } finally {
            restoreGet();
        }
    },
};

export const ShowsEmptyState: Story = {
    args: {
        initialPayments: [],
    },
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
        await expect(
            canvas.getByText("No payments yet. Create your first payment to start tracking payouts."),
        ).toBeVisible();
    },
};
