/**
 * Storybook interaction tests for `/dashboard` protected-route behavior.
 *
 * Covered scenarios:
 * - Loading spinner while auth state is resolving.
 * - Redirect to log-in when user is unauthenticated.
 * - Dashboard page render when user is authenticated.
 */
import { expect, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { RouteComponent as DashboardRouteComponent } from "@/routes/dashboard/index";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";

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

export const LoadingState: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/dashboard",
                initialEntries: ["/dashboard"],
            },
            auth: {
                isLoading: true,
                isLoggedIn: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("Loading")).toBeVisible();
    },
};

export const RedirectsWhenUnauthenticated: Story = {
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
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await waitFor(() => {
            expect(canvas.getByText("Log In Route")).toBeVisible();
        });
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
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Dashboard Page" })).toBeVisible();
    },
};
