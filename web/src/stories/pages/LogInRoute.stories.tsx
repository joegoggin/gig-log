/**
 * Storybook interaction tests for `/auth/log-in` route wrapper behavior.
 *
 * Covered scenarios:
 * - Loading spinner while auth state is resolving.
 * - Redirect to dashboard when user is already authenticated.
 * - Log-in page render when user is logged out.
 */
import { expect, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { RouteComponent as LogInRouteComponent } from "@/routes/auth/log-in/index";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof LogInRouteComponent> = {
    title: "Pages/LogInRoute",
    component: LogInRouteComponent,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/auth/log-in",
                initialEntries: ["/auth/log-in"],
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof LogInRouteComponent>;

export const LoadingState: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/auth/log-in",
                initialEntries: ["/auth/log-in"],
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

export const RedirectsWhenAuthenticated: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/auth/log-in",
                initialEntries: ["/auth/log-in"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: true,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await waitFor(() => {
            expect(canvas.getByText("Dashboard Route")).toBeVisible();
        });
    },
};

export const RendersLogInPageWhenLoggedOut: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/auth/log-in",
                initialEntries: ["/auth/log-in"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Log In" })).toBeVisible();
    },
};
