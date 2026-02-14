/**
 * Storybook interaction tests for `/payments` protected-route behavior.
 *
 * Covered scenarios:
 * - Payments placeholder renders when user is unauthenticated.
 * - Payments placeholder renders when user is authenticated.
 */
import { expect, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { RouteComponent as PaymentsRouteComponent } from "@/routes/_authenticated/payments/index";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof PaymentsRouteComponent> = {
    title: "Pages/PaymentsRoute",
    component: PaymentsRouteComponent,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/payments",
                initialEntries: ["/payments"],
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof PaymentsRouteComponent>;

export const RendersWhenUnauthenticated: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/payments",
                initialEntries: ["/payments"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: false,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }: { canvasElement: HTMLElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Payments" })).toBeVisible();
    },
};

export const RendersWhenAuthenticated: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/payments",
                initialEntries: ["/payments"],
            },
            auth: {
                isLoading: false,
                isLoggedIn: true,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }: { canvasElement: HTMLElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Payments" })).toBeVisible();
    },
};
