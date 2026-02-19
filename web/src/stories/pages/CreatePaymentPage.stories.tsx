/**
 * Storybook interaction tests for Create Payment page behavior.
 *
 * Covered scenarios:
 * - Successful submission sends expected payload, shows success feedback, and navigates to payment detail.
 * - API validation errors are rendered at field level for transfer/date consistency rules.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import CreatePaymentPage from "@/pages/CreatePaymentPage/CreatePaymentPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import {
    createMockApiResponse,
    createValidationAxiosError,
    mockApiGetHandler,
    mockApiPostHandler,
} from "@/test-utils/mockApiClient";

const addNotificationSpy = fn();

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

type StoryLoadedContext = {
    restoreGet: () => void;
};

type PostCall = {
    url: string;
    data: unknown;
};

const meta: Meta<typeof CreatePaymentPage> = {
    title: "Pages/CreatePaymentPage",
    component: CreatePaymentPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/payments/create",
                initialEntries: ["/payments/create"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof CreatePaymentPage>;

export const SubmitsPayloadShowsSuccessAndNavigates: Story = {
    args: {
        preselectedCompanyId: "11111111-1111-1111-1111-111111111111",
    },
    loaders: [
        () => ({
            restoreGet: mockApiGetHandler((url) => {
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
                storyPath: "/payments/create",
                initialEntries: ["/payments/create"],
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
        const postCalls: Array<PostCall> = [];
        const { restoreGet } = loaded as StoryLoadedContext;
        const restorePost = mockApiPostHandler((url, data) => {
            postCalls.push({ url, data });
            return Promise.resolve(
                createMockApiResponse(
                    {
                        payment: {
                            id: "p1",
                        },
                    },
                    201,
                    "Created",
                ),
            );
        });

        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);

            await waitFor(() => {
                expect(canvas.getByRole("option", { name: "Acme Studio" })).toBeVisible();
            });

            await expect(canvas.getByLabelText("Company")).toHaveValue(
                "11111111-1111-1111-1111-111111111111",
            );

            await userEvent.type(canvas.getByPlaceholderText("Total"), "250");
            await userEvent.selectOptions(canvas.getByLabelText("Payout Type"), "paypal");
            await userEvent.type(canvas.getByLabelText("Expected Payout Date"), "2026-03-10");
            await userEvent.type(canvas.getByLabelText("Expected Transfer Date"), "2026-03-12");
            await userEvent.click(canvas.getByLabelText("Payment Received"));
            await userEvent.click(canvas.getByLabelText("Transfer Initiated"));
            await userEvent.click(canvas.getByLabelText("Transfer Received"));
            await userEvent.click(canvas.getByLabelText("Tax Withholdings Covered"));
            await userEvent.click(canvas.getByRole("button", { name: "Create Payment" }));

            await waitFor(() => {
                expect(postCalls).toHaveLength(1);
            });

            await expect(postCalls[0]).toEqual({
                url: "/payments",
                data: {
                    company_id: "11111111-1111-1111-1111-111111111111",
                    total: "250",
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
                title: "Payment Created",
                message: "Your payment has been created successfully.",
            });

            await expect(canvas.getByText("Payment Route")).toBeVisible();
        } finally {
            restoreGet();
            restorePost();
        }
    },
};

export const ShowsValidationErrorsFromApi: Story = {
    args: {
        preselectedCompanyId: "11111111-1111-1111-1111-111111111111",
    },
    loaders: [
        () => ({
            restoreGet: mockApiGetHandler((url) => {
                if (url === "/companies") {
                    return Promise.resolve(createMockApiResponse({ companies: companiesFixture }));
                }

                return Promise.resolve(createMockApiResponse({}));
            }),
        }),
    ],
    play: async ({ canvasElement, loaded }) => {
        const { restoreGet } = loaded as StoryLoadedContext;
        const restorePost = mockApiPostHandler(() => {
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
                expect(canvas.getByRole("option", { name: "Acme Studio" })).toBeVisible();
            });

            await userEvent.type(canvas.getByPlaceholderText("Total"), "100");
            await userEvent.selectOptions(canvas.getByLabelText("Payout Type"), "paypal");
            await userEvent.type(canvas.getByLabelText("Expected Payout Date"), "2026-03-10");
            await userEvent.type(canvas.getByLabelText("Expected Transfer Date"), "2026-03-12");
            await userEvent.click(canvas.getByRole("button", { name: "Create Payment" }));

            const dateValidationMessages = canvas.getAllByText(
                "Expected transfer date cannot be earlier than expected payout date",
            );
            await expect(dateValidationMessages.length).toBe(2);
        } finally {
            restoreGet();
            restorePost();
        }
    },
};
