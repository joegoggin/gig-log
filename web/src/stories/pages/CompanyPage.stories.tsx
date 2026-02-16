/**
 * Storybook interaction tests for Company page behavior.
 *
 * Covered scenarios:
 * - Company details load for the provided company ID.
 * - Not-found responses render the fallback state.
 */
import { expect, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import CompanyPage from "@/pages/CompanyPage/CompanyPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

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

export const ShowsCompanyDetails: Story = {
    args: {
        companyId: "123",
        initialCompany: {
            id: "123",
            user_id: "u1",
            name: "Acme Studio",
            requires_tax_withholdings: true,
            tax_withholding_rate: "30.00",
            created_at: "2026-01-01T00:00:00Z",
            updated_at: "2026-01-02T00:00:00Z",
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
        await expect(canvas.getByText("Tax withholdings: Enabled (30.00%)")).toBeVisible();
    },
};

export const ShowsFallbackWhenCompanyMissing: Story = {
    args: {
        companyId: "123",
        initialCompany: null,
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("This company could not be found.")).toBeVisible();
    },
};
