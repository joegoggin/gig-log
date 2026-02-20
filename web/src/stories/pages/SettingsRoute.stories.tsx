/**
 * Storybook interaction tests for `/settings` protected-route behavior.
 *
 * Covered scenarios:
 * - Unauthenticated users are redirected to the log-in route.
 * - Authenticated users can access the protected app shell for settings routes.
 * - Loading auth state shows a loading indicator.
 */
import { expect, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { RouteComponent as AuthenticatedRouteComponent } from "@/routes/_authenticated";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof AuthenticatedRouteComponent> = {
    title: "Pages/SettingsRoute",
    component: AuthenticatedRouteComponent,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/settings",
                initialEntries: ["/settings"],
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof AuthenticatedRouteComponent>;

export const RedirectsWhenUnauthenticated: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/settings",
                initialEntries: ["/settings"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }: { canvasElement: HTMLElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("Log In Route")).toBeVisible();
    },
};

export const RendersProtectedShellWhenAuthenticated: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/settings",
                initialEntries: ["/settings"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: true,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }: { canvasElement: HTMLElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("button", { name: "Dashboard" })).toBeVisible();
        await expect(canvas.queryByText("Log In Route")).toBeNull();
    },
};

export const RendersNestedSettingsRouteWhenAuthenticated: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/settings/password",
                initialEntries: ["/settings/password"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: true,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }: { canvasElement: HTMLElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("button", { name: "Dashboard" })).toBeVisible();
        await expect(canvas.queryByText("Log In Route")).toBeNull();
    },
};

export const ShowsLoadingState: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/settings",
                initialEntries: ["/settings"],
            },
            auth: {
                isLoading: true,
                isLoggedIn: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }: { canvasElement: HTMLElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("Loading")).toBeVisible();
    },
};
