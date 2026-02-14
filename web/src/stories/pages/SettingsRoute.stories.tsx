/**
 * Storybook interaction tests for `/settings` protected-route behavior.
 *
 * Covered scenarios:
 * - Redirect to log-in when user is unauthenticated.
 * - Settings placeholder renders when user is authenticated.
 */
import { expect, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { RouteComponent as SettingsRouteComponent } from "@/routes/settings/index";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof SettingsRouteComponent> = {
    title: "Pages/SettingsRoute",
    component: SettingsRouteComponent,
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
type Story = StoryObj<typeof SettingsRouteComponent>;

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
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await waitFor(() => {
            expect(canvas.getByText("Log In Route")).toBeVisible();
        });
    },
};

export const RendersWhenAuthenticated: Story = {
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
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Settings" })).toBeVisible();
    },
};
