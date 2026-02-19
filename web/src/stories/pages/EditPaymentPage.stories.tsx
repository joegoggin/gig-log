/**
 * Storybook interaction tests for Edit Payment page behavior.
 *
 * Covered scenarios:
 * - Existing payment data pre-fills fields, submits update payloads, and navigates to payment details.
 * - Validation errors from update requests render on date-related fields.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import EditPaymentPage from "@/pages/EditPaymentPage/EditPaymentPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import {
    createMockApiResponse,
    createValidationAxiosError,
    mockApiGetHandler,
    mockApiPutHandler,
} from "@/test-utils/mockApiClient";

const addNotificationSpy = fn();

type PutCall = {
    url: string;
    data: unknown;
};

type StoryLoadedContext = {
    restoreGet: () => void;
};

const companiesFixture = [
    {
        id: "11111111-1111-1111-1111-111111111111",
        user_id: "u1",
        name: "Acme Studio",
        requires_tax_withholdings: false,
        tax_withholding_rate: null,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
    },
    {
        id: "22222222-2222-2222-2222-222222222222",
        user_id: "u1",
        name: "Nova Labs",
        requires_tax_withholdings: false,
        tax_withholding_rate: null,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
    },
];

const createPaymentDetailResponse = () =>
    createMockApiResponse({
        payment: {
            id: "p1",
            user_id: "u1",
            company_id: "11111111-1111-1111-1111-111111111111",
            total: "250.00",
            payout_type: "paypal",
            expected_payout_date: "2026-03-10",
            expected_transfer_date: "2026-03-12",
            transfer_initiated: true,
            payment_received: true,
            transfer_received: false,
            tax_withholdings_covered: false,
            created_at: "2026-03-01T00:00:00Z",
            updated_at: "2026-03-02T00:00:00Z",
        },
    });

const meta: Meta<typeof EditPaymentPage> = {
    title: "Pages/EditPaymentPage",
    component: EditPaymentPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    args: {
        paymentId: "p1",
    },
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/payments/p1/edit",
                initialEntries: ["/payments/p1/edit"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof EditPaymentPage>;

export const PrefillsAndSubmitsUpdate: Story = {
    loaders: [
        () => ({
            restoreGet: mockApiGetHandler((url) => {
                if (url === "/payments/p1") {
                    return Promise.resolve(createPaymentDetailResponse());
                }

                if (url === "/companies") {
                    return Promise.resolve(createMockApiResponse({ companies: companiesFixture }));
                }

                return Promise.resolve(createMockApiResponse({}));
            }),
        }),
    ],
    parameters: {
        storyTest: {
            router: {
                storyPath: "/payments/p1/edit",
                initialEntries: ["/payments/p1/edit"],
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
        const putCalls: Array<PutCall> = [];
        const { restoreGet } = loaded as StoryLoadedContext;
        const restorePut = mockApiPutHandler((url, data) => {
            putCalls.push({ url, data });
            return Promise.resolve(
                createMockApiResponse({
                    payment: {
                        id: "p1",
                        user_id: "u1",
                        company_id: "22222222-2222-2222-2222-222222222222",
                        total: "375",
                        payout_type: "paypal",
                        expected_payout_date: "2026-03-10",
                        expected_transfer_date: "2026-03-12",
                        transfer_initiated: true,
                        payment_received: true,
                        transfer_received: true,
                        tax_withholdings_covered: true,
                        created_at: "2026-03-01T00:00:00Z",
                        updated_at: "2026-03-03T00:00:00Z",
                    },
                }),
            );
        });

        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);

            await waitFor(() => {
                expect(canvas.getByDisplayValue("250.00")).toBeVisible();
            });

            await expect(canvas.getByLabelText("Company")).toHaveValue(
                "11111111-1111-1111-1111-111111111111",
            );
            await expect(canvas.getByLabelText("Payout Type")).toHaveValue("paypal");
            await expect(canvas.getByLabelText("Expected Payout Date")).toHaveValue("2026-03-10");
            await expect(canvas.getByLabelText("Expected Transfer Date")).toHaveValue("2026-03-12");
            await expect(canvas.getByLabelText("Payment Received")).toBeChecked();
            await expect(canvas.getByLabelText("Transfer Initiated")).toBeChecked();
            await expect(canvas.getByLabelText("Transfer Received")).not.toBeChecked();

            await userEvent.selectOptions(
                canvas.getByLabelText("Company"),
                "22222222-2222-2222-2222-222222222222",
            );
            await userEvent.clear(canvas.getByPlaceholderText("Total"));
            await userEvent.type(canvas.getByPlaceholderText("Total"), "375");
            await userEvent.click(canvas.getByLabelText("Transfer Received"));
            await userEvent.click(canvas.getByLabelText("Tax Withholdings Covered"));
            await userEvent.click(canvas.getByRole("button", { name: "Save Payment" }));

            await waitFor(() => {
                expect(putCalls).toHaveLength(1);
            });
            await expect(putCalls[0]).toEqual({
                url: "/payments/p1",
                data: {
                    company_id: "22222222-2222-2222-2222-222222222222",
                    total: "375",
                    payout_type: "paypal",
                    expected_payout_date: "2026-03-10",
                    expected_transfer_date: "2026-03-12",
                    transfer_initiated: true,
                    payment_received: true,
                    transfer_received: true,
                    tax_withholdings_covered: true,
                },
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Payment Updated",
                message: "Payment details were updated successfully.",
            });
            await expect(canvas.getByText("Payment Route")).toBeVisible();
        } finally {
            restoreGet();
            restorePut();
        }
    },
};

export const ShowsValidationErrorsFromApi: Story = {
    loaders: [
        () => ({
            restoreGet: mockApiGetHandler((url) => {
                if (url === "/payments/p1") {
                    return Promise.resolve(createPaymentDetailResponse());
                }

                if (url === "/companies") {
                    return Promise.resolve(createMockApiResponse({ companies: companiesFixture }));
                }

                return Promise.resolve(createMockApiResponse({}));
            }),
        }),
    ],
    play: async ({ canvasElement, loaded }) => {
        const { restoreGet } = loaded as StoryLoadedContext;
        const restorePut = mockApiPutHandler(() => {
            return Promise.reject(
                createValidationAxiosError([
                    {
                        field: "expected_transfer_date_order",
                        message:
                            "Expected transfer date cannot be earlier than expected payout date",
                    },
                ]),
            );
        });

        try {
            const canvas = within(canvasElement);

            await waitFor(() => {
                expect(canvas.getByDisplayValue("250.00")).toBeVisible();
            });

            await userEvent.click(canvas.getByRole("button", { name: "Save Payment" }));

            const dateValidationMessages = canvas.getAllByText(
                "Expected transfer date cannot be earlier than expected payout date",
            );
            await expect(dateValidationMessages.length).toBe(2);
        } finally {
            restoreGet();
            restorePut();
        }
    },
};
