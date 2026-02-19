/**
 * Storybook interaction tests for Edit Job page behavior.
 *
 * Covered scenarios:
 * - Existing job data pre-fills fields, submits update payloads, and navigates to job details.
 * - Validation errors from updates render on payment-type-specific fields.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import EditJobPage from "@/pages/EditJobPage/EditJobPage";
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

const createJobDetailResponse = () =>
    createMockApiResponse({
        job: {
            id: "123",
            company_id: "11111111-1111-1111-1111-111111111111",
            user_id: "u1",
            title: "Website Maintenance",
            payment_type: "hourly",
            number_of_payouts: null,
            payout_amount: null,
            hourly_rate: "55.50",
            created_at: "2026-01-01T00:00:00Z",
            updated_at: "2026-01-01T00:00:00Z",
        },
    });

const meta: Meta<typeof EditJobPage> = {
    title: "Pages/EditJobPage",
    component: EditJobPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    args: {
        jobId: "123",
    },
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/jobs/123/edit",
                initialEntries: ["/jobs/123/edit"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof EditJobPage>;

export const PrefillsAndSubmitsUpdate: Story = {
    loaders: [
        () => ({
            restoreGet: mockApiGetHandler((url) => {
                if (url === "/jobs/123") {
                    return Promise.resolve(createJobDetailResponse());
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
                storyPath: "/jobs/123/edit",
                initialEntries: ["/jobs/123/edit"],
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
                    job: {
                        id: "123",
                        company_id: "22222222-2222-2222-2222-222222222222",
                        user_id: "u1",
                        title: "Website Retainer",
                        payment_type: "payouts",
                        number_of_payouts: 2,
                        payout_amount: "300",
                        hourly_rate: null,
                        created_at: "2026-01-01T00:00:00Z",
                        updated_at: "2026-01-02T00:00:00Z",
                    },
                }),
            );
        });

        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await waitFor(() => {
                expect(canvas.getByDisplayValue("Website Maintenance")).toBeVisible();
            });

            await expect(canvas.getByLabelText("Company")).toHaveValue(
                "11111111-1111-1111-1111-111111111111",
            );
            await expect(canvas.getByLabelText("Payment Type")).toHaveValue("hourly");
            await expect(canvas.getByPlaceholderText("Hourly Rate")).toBeVisible();

            await userEvent.clear(canvas.getByPlaceholderText("Job Title"));
            await userEvent.type(canvas.getByPlaceholderText("Job Title"), "Website Retainer");
            await userEvent.selectOptions(
                canvas.getByLabelText("Company"),
                "22222222-2222-2222-2222-222222222222",
            );
            await userEvent.selectOptions(canvas.getByLabelText("Payment Type"), "payouts");
            await userEvent.type(canvas.getByPlaceholderText("Number of Payouts"), "2");
            await userEvent.type(canvas.getByPlaceholderText("Payout Amount"), "300");
            await userEvent.click(canvas.getByRole("button", { name: "Save Job" }));

            await waitFor(() => {
                expect(putCalls).toHaveLength(1);
            });
            await expect(putCalls[0]).toEqual({
                url: "/jobs/123",
                data: {
                    company_id: "22222222-2222-2222-2222-222222222222",
                    title: "Website Retainer",
                    payment_type: "payouts",
                    number_of_payouts: 2,
                    payout_amount: "300",
                    hourly_rate: null,
                },
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Job Updated",
                message: "Job details were updated successfully.",
            });
            await expect(canvas.getByText("Job Route")).toBeVisible();
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
                if (url === "/jobs/123") {
                    return Promise.resolve(createJobDetailResponse());
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
        const restorePut = mockApiPutHandler(() =>
            Promise.reject(
                createValidationAxiosError([
                    {
                        field: "payout_fields_required",
                        message:
                            "Number of payouts and payout amount are required when payment type is payouts",
                    },
                ]),
            ),
        );

        try {
            const canvas = within(canvasElement);
            await waitFor(() => {
                expect(canvas.getByDisplayValue("Website Maintenance")).toBeVisible();
            });

            await userEvent.selectOptions(canvas.getByLabelText("Payment Type"), "payouts");
            await userEvent.type(canvas.getByPlaceholderText("Number of Payouts"), "2");
            await userEvent.type(canvas.getByPlaceholderText("Payout Amount"), "300");
            await userEvent.click(canvas.getByRole("button", { name: "Save Job" }));

            const payoutValidationMessages = canvas.getAllByText(
                "Number of payouts and payout amount are required when payment type is payouts",
            );
            await expect(payoutValidationMessages.length).toBe(2);
        } finally {
            restoreGet();
            restorePut();
        }
    },
};
