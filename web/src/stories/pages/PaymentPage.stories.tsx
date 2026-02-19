/**
 * Storybook interaction tests for Payment page behavior.
 *
 * Covered scenarios:
 * - Payment details render with payout, date, and status information.
 * - Back navigation returns to the payments list route.
 * - API 404 responses show not-found fallback UI and notifications.
 * - API failures show retry UI and recover on successful retry.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import type { Payment } from "@/types/models/Payment";
import { NotificationType } from "@/components/core/Notification/Notification";
import PaymentPage from "@/pages/PaymentPage/PaymentPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import {
    createAxiosErrorResponse,
    createMockApiResponse,
    mockApiGetHandler,
} from "@/test-utils/mockApiClient";

const addNotificationSpy = fn();

const transferPaymentFixture: Payment = {
    id: "p1",
    user_id: "u1",
    company_id: "c1",
    total: "500.00",
    payout_type: "paypal",
    expected_payout_date: "2026-03-10",
    expected_transfer_date: "2026-03-12",
    transfer_initiated: true,
    payment_received: true,
    transfer_received: true,
    tax_withholdings_covered: false,
    created_at: "2026-03-01T00:00:00Z",
    updated_at: "2026-03-02T00:00:00Z",
};

const directDepositFixture: Payment = {
    ...transferPaymentFixture,
    id: "p2",
    payout_type: "direct_deposit",
    transfer_initiated: false,
    transfer_received: false,
};

type StoryLoadedContext = {
    restoreGet: () => void;
};

const meta: Meta<typeof PaymentPage> = {
    title: "Pages/PaymentPage",
    component: PaymentPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    args: {
        paymentId: "p1",
    },
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/payments/p1",
                initialEntries: ["/payments/p1"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof PaymentPage>;

export const ShowsPaymentDetailsAndRoutesBack: Story = {
    args: {
        paymentId: "p1",
        initialPayment: transferPaymentFixture,
    },
    parameters: {
        storyTest: {
            router: {
                storyPath: "/payments/p1",
                initialEntries: ["/payments/p1"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Payment: $500.00" })).toBeVisible();
        await expect(canvas.getByText("Payout type")).toBeVisible();
        await expect(canvas.getByText("PayPal")).toBeVisible();
        await expect(canvas.getByText("Expected payout")).toBeVisible();
        await expect(canvas.getByText("Expected transfer")).toBeVisible();
        await expect(canvas.getByText("Payment received")).toBeVisible();
        await expect(canvas.getByText("Transfer initiated")).toBeVisible();
        await expect(canvas.getByText("Transfer received")).toBeVisible();
        await expect(canvas.getByText("Tax withholdings covered")).toBeVisible();

        await userEvent.click(canvas.getByRole("button", { name: "Back to Payments" }));
        await expect(canvas.getByText("Payments Route")).toBeVisible();
    },
};

export const HidesTransferFieldsForDirectDeposit: Story = {
    args: {
        paymentId: "p2",
        initialPayment: directDepositFixture,
    },
    parameters: {
        storyTest: {
            router: {
                storyPath: "/payments/p2",
                initialEntries: ["/payments/p2"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("Direct Deposit")).toBeVisible();
        await expect(canvas.queryByText("Expected transfer")).toBeNull();
        await expect(canvas.queryByText("Transfer initiated")).toBeNull();
        await expect(canvas.queryByText("Transfer received")).toBeNull();
    },
};

export const ShowsNotFoundFromApiResponse: Story = {
    args: {
        paymentId: "404",
    },
    loaders: [
        () => {
            addNotificationSpy.mockClear();

            return {
                restoreGet: mockApiGetHandler((url) => {
                    if (url === "/payments/404") {
                        return Promise.reject(
                            createAxiosErrorResponse(
                                {
                                    error: {
                                        code: "not_found",
                                        message: "Payment not found",
                                    },
                                },
                                404,
                                "Not Found",
                            ),
                        );
                    }

                    return Promise.resolve(createMockApiResponse({}));
                }),
            };
        },
    ],
    parameters: {
        storyTest: {
            router: {
                storyPath: "/payments/404",
                initialEntries: ["/payments/404"],
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
                expect(canvas.getByText("This payment could not be found.")).toBeVisible();
            });

            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.ERROR,
                title: "Payment Not Found",
                message: "Unable to load the requested payment.",
            });
        } finally {
            restoreGet();
        }
    },
};

export const RetriesAfterApiFailure: Story = {
    args: {
        paymentId: "500",
    },
    loaders: [
        () => {
            addNotificationSpy.mockClear();
            let requestCount = 0;

            return {
                restoreGet: mockApiGetHandler((url) => {
                    if (url === "/payments/500") {
                        requestCount += 1;

                        if (requestCount === 1) {
                            return Promise.reject(
                                createAxiosErrorResponse({
                                    error: {
                                        code: "server_error",
                                        message: "Unable to load payment",
                                    },
                                }),
                            );
                        }

                        return Promise.resolve(
                            createMockApiResponse({
                                payment: {
                                    ...transferPaymentFixture,
                                    id: "500",
                                },
                            }),
                        );
                    }

                    return Promise.resolve(createMockApiResponse({}));
                }),
            };
        },
    ],
    parameters: {
        storyTest: {
            router: {
                storyPath: "/payments/500",
                initialEntries: ["/payments/500"],
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
                expect(canvas.getByText("Unable to load this payment right now.")).toBeVisible();
            });

            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.ERROR,
                title: "Payment Unavailable",
                message: "Unable to load payment details right now.",
            });

            await userEvent.click(canvas.getByRole("button", { name: "Retry" }));

            await waitFor(() => {
                expect(canvas.getByRole("heading", { name: "Payment: $500.00" })).toBeVisible();
            });
        } finally {
            restoreGet();
        }
    },
};
