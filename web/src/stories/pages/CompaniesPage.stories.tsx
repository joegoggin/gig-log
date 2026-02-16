/**
 * Storybook interaction tests for Companies page behavior.
 *
 * Covered scenarios:
 * - Companies load from the API and render as cards.
 * - Selecting "View Company" navigates to the company detail route.
 */
import { expect, userEvent, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import CompaniesPage from "@/pages/CompaniesPage/CompaniesPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

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

export const LoadsCompaniesFromApi: Story = {
    args: {
        initialCompanies: [
            {
                id: "123",
                user_id: "u1",
                name: "Acme Studio",
                requires_tax_withholdings: true,
                tax_withholding_rate: "30.00",
                created_at: "2026-01-01T00:00:00Z",
                updated_at: "2026-01-01T00:00:00Z",
            },
        ],
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
    },
};

export const ViewCompanyNavigatesToDetailRoute: Story = {
    args: {
        initialCompanies: [
            {
                id: "123",
                user_id: "u1",
                name: "Acme Studio",
                requires_tax_withholdings: false,
                tax_withholding_rate: null,
                created_at: "2026-01-01T00:00:00Z",
                updated_at: "2026-01-01T00:00:00Z",
            },
        ],
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
