/**
 * Storybook interaction tests for `/auth/set-password` protected-route behavior.
 *
 * Covered scenarios:
 * - Loading spinner while auth state is resolving.
 * - Redirect to log-in when user is unauthenticated.
 * - Set-password page render when user is authenticated.
 */
import { expect, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { RouteComponent as SetPasswordRouteComponent } from "@/routes/auth/set-password/index";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";

const meta: Meta<typeof SetPasswordRouteComponent> = {
    title: "Pages/SetPasswordRoute",
    component: SetPasswordRouteComponent,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/auth/set-password",
                initialEntries: ["/auth/set-password"],
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof SetPasswordRouteComponent>;

export const LoadingState: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/auth/set-password",
                initialEntries: ["/auth/set-password"],
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
                storyPath: "/auth/set-password",
                initialEntries: ["/auth/set-password"],
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

export const RendersSetPasswordWhenAuthenticated: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/auth/set-password",
                initialEntries: ["/auth/set-password"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: true,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Set Password" })).toBeVisible();
    },
};
