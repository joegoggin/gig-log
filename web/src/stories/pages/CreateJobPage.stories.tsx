/**
 * Storybook interaction tests for Create Job page behavior.
 *
 * Covered scenarios:
 * - Successful submission sends expected payload, shows success feedback, and resets the form.
 * - Validation errors from the API are rendered at field level.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import CreateJobPage from "@/pages/CreateJobPage/CreateJobPage";
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

const meta: Meta<typeof CreateJobPage> = {
    title: "Pages/CreateJobPage",
    component: CreateJobPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/jobs/create",
                initialEntries: ["/jobs/create"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof CreateJobPage>;

export const SubmitsPayloadShowsSuccessAndResetsForm: Story = {
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
                storyPath: "/jobs/create",
                initialEntries: ["/jobs/create"],
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
                        job: {
                            id: "j1",
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
            await userEvent.type(canvas.getByPlaceholderText("Job Title"), "Website Retainer");
            await userEvent.selectOptions(canvas.getByLabelText("Payment Type"), "payouts");
            await userEvent.type(canvas.getByPlaceholderText("Number of Payouts"), "3");
            await userEvent.type(canvas.getByPlaceholderText("Payout Amount"), "250.00");
            await userEvent.click(canvas.getByRole("button", { name: "Create Job" }));

            await waitFor(() => {
                expect(postCalls).toHaveLength(1);
            });
            await expect(postCalls[0]).toEqual({
                url: "/jobs",
                data: {
                    company_id: "11111111-1111-1111-1111-111111111111",
                    title: "Website Retainer",
                    payment_type: "payouts",
                    number_of_payouts: 3,
                    payout_amount: "250",
                    hourly_rate: null,
                },
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Job Created",
                message: "Your job has been created successfully.",
            });
            await expect(canvas.getByPlaceholderText("Job Title")).toHaveValue("");
            await expect(canvas.getByLabelText("Company")).toHaveValue(
                "11111111-1111-1111-1111-111111111111",
            );
            await expect(canvas.getByLabelText("Payment Type")).toHaveValue("hourly");
            await expect(canvas.queryByPlaceholderText("Number of Payouts")).toBeNull();
            await expect(canvas.queryByPlaceholderText("Payout Amount")).toBeNull();
            await expect(canvas.getByPlaceholderText("Hourly Rate")).toBeVisible();
        } finally {
            restoreGet();
            restorePost();
        }
    },
};

export const ShowsValidationErrorsFromApi: Story = {
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
                        field: "payout_fields_required",
                        message:
                            "Number of payouts and payout amount are required when payment type is payouts",
                    },
                ]),
            );
        });

        try {
            const canvas = within(canvasElement);
            await waitFor(() => {
                expect(canvas.getByRole("option", { name: "Acme Studio" })).toBeVisible();
            });

            await userEvent.type(canvas.getByPlaceholderText("Job Title"), "Retainer Plan");
            await userEvent.selectOptions(
                canvas.getByLabelText("Company"),
                "11111111-1111-1111-1111-111111111111",
            );
            await userEvent.selectOptions(canvas.getByLabelText("Payment Type"), "payouts");
            await userEvent.type(canvas.getByPlaceholderText("Number of Payouts"), "2");
            await userEvent.type(canvas.getByPlaceholderText("Payout Amount"), "150.00");
            await userEvent.click(canvas.getByRole("button", { name: "Create Job" }));

            const payoutValidationMessages = canvas.getAllByText(
                "Number of payouts and payout amount are required when payment type is payouts",
            );
            await expect(payoutValidationMessages.length).toBe(2);
        } finally {
            restoreGet();
            restorePost();
        }
    },
};
