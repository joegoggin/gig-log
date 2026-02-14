/**
 * Storybook interaction tests for `/dashboard` protected-route behavior.
 *
 * Covered scenarios:
 * - Dashboard page render when user is unauthenticated.
 * - Dashboard page render when user is authenticated.
 */
import { expect, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { RouteComponent as DashboardRouteComponent } from "@/routes/_authenticated/dashboard/index";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof DashboardRouteComponent> = {
    title: "Pages/DashboardRoute",
    component: DashboardRouteComponent,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/dashboard",
                initialEntries: ["/dashboard"],
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof DashboardRouteComponent>;

export const RendersWhenUnauthenticated: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/dashboard",
                initialEntries: ["/dashboard"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }: { canvasElement: HTMLElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Dashboard" })).toBeVisible();
    },
};

export const RendersDashboardWhenAuthenticated: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/dashboard",
                initialEntries: ["/dashboard"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: true,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }: { canvasElement: HTMLElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Dashboard" })).toBeVisible();
    },
};
