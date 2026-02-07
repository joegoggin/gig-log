/**
 * Storybook interaction tests for `/auth/sign-up` route wrapper behavior.
 *
 * Covered scenarios:
 * - Loading spinner while auth state is resolving.
 * - Redirect to dashboard when user is already authenticated.
 * - Sign-up page render when user is logged out.
 */
import { expect, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { RouteComponent as SignUpRouteComponent } from "@/routes/auth/sign-up/index";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof SignUpRouteComponent> = {
    title: "Pages/SignUpRoute",
    component: SignUpRouteComponent,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/auth/sign-up",
                initialEntries: ["/auth/sign-up"],
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof SignUpRouteComponent>;

export const LoadingState: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/auth/sign-up",
                initialEntries: ["/auth/sign-up"],
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
                storyPath: "/auth/sign-up",
                initialEntries: ["/auth/sign-up"],
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

export const RendersSignUpPageWhenLoggedOut: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/auth/sign-up",
                initialEntries: ["/auth/sign-up"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Sign Up" })).toBeVisible();
    },
};
