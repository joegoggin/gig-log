/**
 * Storybook interaction tests for Edit Company page behavior.
 *
 * Covered scenarios:
 * - Existing company data pre-fills, update payload is submitted, and navigation occurs.
 * - Validation errors from update requests render at field level.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import EditCompanyPage from "@/pages/EditCompanyPage/EditCompanyPage";
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

const meta: Meta<typeof EditCompanyPage> = {
    title: "Pages/EditCompanyPage",
    component: EditCompanyPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    args: {
        companyId: "123",
    },
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/companies/123/edit",
                initialEntries: ["/companies/123/edit"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof EditCompanyPage>;

export const PrefillsAndSubmitsUpdate: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/companies/123/edit",
                initialEntries: ["/companies/123/edit"],
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
        const putCalls: Array<PutCall> = [];
        const restoreGet = mockApiGetHandler(async (url) => {
            if (url === "/companies/123") {
                return createMockApiResponse({
                    company: {
                        id: "123",
                        user_id: "u1",
                        name: "Acme Studio",
                        requires_tax_withholdings: false,
                        tax_withholding_rate: null,
                        created_at: "2026-01-01T00:00:00Z",
                        updated_at: "2026-01-01T00:00:00Z",
                    },
                });
            }

            return createMockApiResponse({});
        });
        const restorePut = mockApiPutHandler(async (url, data) => {
            putCalls.push({ url, data });
            return createMockApiResponse({
                company: {
                    id: "123",
                    user_id: "u1",
                    name: "Acme Studio",
                    requires_tax_withholdings: false,
                    tax_withholding_rate: null,
                    created_at: "2026-01-01T00:00:00Z",
                    updated_at: "2026-01-02T00:00:00Z",
                },
            });
        });

        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await waitFor(() => {
                expect(canvas.getByDisplayValue("Acme Studio")).toBeVisible();
            });

            await userEvent.click(canvas.getByRole("button", { name: "Save Company" }));

            await waitFor(() => {
                expect(putCalls).toHaveLength(1);
            });
            await expect(putCalls[0]).toEqual({
                url: "/companies/123",
                data: {
                    name: "Acme Studio",
                    requires_tax_withholdings: false,
                    tax_withholding_rate: null,
                },
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Company Updated",
                message: "Company details were updated successfully.",
            });
            await expect(canvas.getByText("Company Route")).toBeVisible();
        } finally {
            restoreGet();
            restorePut();
        }
    },
};

export const ShowsValidationErrorsFromApi: Story = {
    play: async ({ canvasElement }) => {
        const restoreGet = mockApiGetHandler(async () =>
            createMockApiResponse({
                company: {
                    id: "123",
                    user_id: "u1",
                    name: "Acme Studio",
                    requires_tax_withholdings: false,
                    tax_withholding_rate: null,
                    created_at: "2026-01-01T00:00:00Z",
                    updated_at: "2026-01-01T00:00:00Z",
                },
            }),
        );
        const restorePut = mockApiPutHandler(async () => {
            throw createValidationAxiosError([
                { field: "name", message: "Company name is required" },
            ]);
        });

        try {
            const canvas = within(canvasElement);
            await waitFor(() => {
                expect(canvas.getByDisplayValue("Acme Studio")).toBeVisible();
            });

            await userEvent.clear(canvas.getByPlaceholderText("Company Name"));
            await userEvent.click(canvas.getByRole("button", { name: "Save Company" }));

            await expect(canvas.getByText("Company name is required")).toBeVisible();
        } finally {
            restoreGet();
            restorePut();
        }
    },
};
