/**
 * Storybook interaction tests for Company page behavior.
 *
 * Covered scenarios:
 * - Company summary, jobs, and payments render from detail payloads.
 * - Job view actions navigate to the job-detail route.
 * - Non-functional edit/delete icon actions still render for each list item.
 * - Jobs pagination appends additional records when loading more.
 * - Payments pagination appends additional records when loading more.
 * - Not-found responses render the fallback state.
 */
import { expect, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import CompanyPage from "@/pages/CompanyPage/CompanyPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import { createMockApiResponse, mockApiGetHandler } from "@/test-utils/mockApiClient";

const meta: Meta<typeof CompanyPage> = {
    title: "Pages/CompanyPage",
    component: CompanyPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    args: {
        companyId: "123",
    },
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/companies/123",
                initialEntries: ["/companies/123"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof CompanyPage>;

export const ShowsCompanyDetailsAndPayments: Story = {
    args: {
        companyId: "123",
        initialCompanyDetail: {
            company: {
                id: "123",
                user_id: "u1",
                name: "Acme Studio",
                requires_tax_withholdings: true,
                tax_withholding_rate: "30.00",
                created_at: "2026-01-01T00:00:00Z",
                updated_at: "2026-01-02T00:00:00Z",
                payment_total: "250.00",
                hours: "4h 30m",
            },
            paginated_jobs: [
                { id: "j1", title: "Website Redesign" },
                { id: "j2", title: "Landing Page Copy" },
            ],
            jobs_has_more: false,
            paginated_payments: [
                {
                    id: "p1",
                    total: "100.00",
                    payout_type: "paypal",
                    payment_received: true,
                    transfer_received: true,
                },
            ],
            payments_has_more: false,
        },
    },
    parameters: {
        storyTest: {
            router: {
                storyPath: "/companies/123",
                initialEntries: ["/companies/123"],
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
        await expect(canvas.getByRole("button", { name: "Add Job" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Add Payment" })).toBeVisible();
        await expect(canvas.getByText("Tax withholding")).toBeVisible();
        await expect(canvas.getByText(/30\.00\s*%/)).toBeVisible();
        await expect(canvas.getByText("$250.00")).toBeVisible();
        await expect(canvas.getByText("4h 30m")).toBeVisible();
        await expect(canvas.getByText("Website Redesign")).toBeVisible();
        await expect(canvas.getByText("Payout Type: paypal")).toBeVisible();
        await expect(canvas.getByRole("button", { name: "View Website Redesign" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "View Landing Page Copy" })).toBeVisible();
        await expect(canvas.getAllByLabelText(/view .* action \(coming soon\)/i).length).toBe(1);
        await expect(canvas.getAllByLabelText(/edit .* action \(coming soon\)/i).length).toBe(3);
        await expect(canvas.getAllByLabelText(/delete .* action \(coming soon\)/i).length).toBe(3);
    },
};

export const RoutesJobViewActionToJobDetail: Story = {
    args: {
        companyId: "123",
        initialCompanyDetail: {
            company: {
                id: "123",
                user_id: "u1",
                name: "Acme Studio",
                requires_tax_withholdings: false,
                tax_withholding_rate: null,
                created_at: "2026-01-01T00:00:00Z",
                updated_at: "2026-01-02T00:00:00Z",
                payment_total: "250.00",
                hours: "4h 30m",
            },
            paginated_jobs: [{ id: "j1", title: "Website Redesign" }],
            jobs_has_more: false,
            paginated_payments: [],
            payments_has_more: false,
        },
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await userEvent.click(canvas.getByRole("button", { name: "View Website Redesign" }));
        await expect(canvas.getByText("Job Route")).toBeVisible();
    },
};

export const LoadsMoreJobsWhenAvailable: Story = {
    args: {
        companyId: "123",
        initialCompanyDetail: {
            company: {
                id: "123",
                user_id: "u1",
                name: "Acme Studio",
                requires_tax_withholdings: false,
                tax_withholding_rate: null,
                created_at: "2026-01-01T00:00:00Z",
                updated_at: "2026-01-02T00:00:00Z",
                payment_total: "250.00",
                hours: "4h 30m",
            },
            paginated_jobs: [{ id: "j1", title: "Website Redesign" }],
            jobs_has_more: true,
            paginated_payments: [],
            payments_has_more: false,
        },
    },
    play: async ({ canvasElement }) => {
        const restoreGet = mockApiGetHandler((url) => {
            if (url === "/companies/123?jobs_page=2") {
                return Promise.resolve(
                    createMockApiResponse({
                        company: {
                            id: "123",
                            user_id: "u1",
                            name: "Acme Studio",
                            requires_tax_withholdings: false,
                            tax_withholding_rate: null,
                            created_at: "2026-01-01T00:00:00Z",
                            updated_at: "2026-01-02T00:00:00Z",
                            payment_total: "250.00",
                            hours: "4h 30m",
                        },
                        paginated_jobs: [{ id: "j2", title: "Marketing Retainer" }],
                        jobs_has_more: false,
                        paginated_payments: [],
                        payments_has_more: false,
                    }),
                );
            }

            return Promise.resolve(createMockApiResponse({}));
        });

        try {
            const canvas = within(canvasElement);
            await expect(canvas.getByText("Website Redesign")).toBeVisible();

            await userEvent.click(canvas.getByRole("button", { name: "Load More Jobs" }));

            await waitFor(() => {
                expect(canvas.getByText("Marketing Retainer")).toBeVisible();
            });
        } finally {
            restoreGet();
        }
    },
};

export const LoadsMorePaymentsWhenAvailable: Story = {
    args: {
        companyId: "123",
        initialCompanyDetail: {
            company: {
                id: "123",
                user_id: "u1",
                name: "Acme Studio",
                requires_tax_withholdings: false,
                tax_withholding_rate: null,
                created_at: "2026-01-01T00:00:00Z",
                updated_at: "2026-01-02T00:00:00Z",
                payment_total: "250.00",
                hours: "4h 30m",
            },
            paginated_jobs: [],
            jobs_has_more: false,
            paginated_payments: [
                {
                    id: "p1",
                    total: "100.00",
                    payout_type: "paypal",
                    payment_received: true,
                    transfer_received: true,
                },
            ],
            payments_has_more: true,
        },
    },
    play: async ({ canvasElement }) => {
        const restoreGet = mockApiGetHandler((url) => {
            if (url === "/companies/123?payments_page=2") {
                return Promise.resolve(
                    createMockApiResponse({
                        company: {
                            id: "123",
                            user_id: "u1",
                            name: "Acme Studio",
                            requires_tax_withholdings: false,
                            tax_withholding_rate: null,
                            created_at: "2026-01-01T00:00:00Z",
                            updated_at: "2026-01-02T00:00:00Z",
                            payment_total: "250.00",
                            hours: "4h 30m",
                        },
                        paginated_jobs: [],
                        jobs_has_more: false,
                        paginated_payments: [
                            {
                                id: "p2",
                                total: "150.00",
                                payout_type: "cash",
                                payment_received: true,
                                transfer_received: false,
                            },
                        ],
                        payments_has_more: false,
                    }),
                );
            }

            return Promise.resolve(createMockApiResponse({}));
        });

        try {
            const canvas = within(canvasElement);
            await expect(canvas.getByText("Total: $100.00")).toBeVisible();

            await userEvent.click(canvas.getByRole("button", { name: "Load More Payments" }));

            await waitFor(() => {
                expect(canvas.getByText("Total: $150.00")).toBeVisible();
            });
        } finally {
            restoreGet();
        }
    },
};

export const ShowsFallbackWhenCompanyMissing: Story = {
    args: {
        companyId: "123",
        initialCompanyDetail: null,
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("This company could not be found.")).toBeVisible();
    },
};
