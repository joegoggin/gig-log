/**
 * Storybook interaction tests for `/companies` protected-route behavior.
 *
 * Covered scenarios:
 * - Companies placeholder renders when user is unauthenticated.
 * - Companies placeholder renders when user is authenticated.
 */
import { expect, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { RouteComponent as CompaniesRouteComponent } from "@/routes/_authenticated/companies/index";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof CompaniesRouteComponent> = {
    title: "Pages/CompaniesRoute",
    component: CompaniesRouteComponent,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/companies",
                initialEntries: ["/companies"],
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof CompaniesRouteComponent>;

export const RendersWhenUnauthenticated: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/companies",
                initialEntries: ["/companies"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }: { canvasElement: HTMLElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Companies" })).toBeVisible();
    },
};

export const RendersWhenAuthenticated: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/companies",
                initialEntries: ["/companies"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: true,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }: { canvasElement: HTMLElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Companies" })).toBeVisible();
    },
};
