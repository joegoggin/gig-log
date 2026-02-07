/**
 * Storybook interaction tests for Home page behavior.
 *
 * Covered scenarios:
 * - Logged-out users see sign-up/log-in actions and can navigate to sign-up.
 * - Logged-in users only see dashboard CTA and can navigate to dashboard.
 */
import { expect, userEvent, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import HomePage from "@/pages/HomePage/HomePage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof HomePage> = {
    title: "Pages/HomePage",
    component: HomePage,
    tags: ["autodocs"],
    decorators: [
        withMemoryRouter,
        withAppProviders,
    ],
    parameters: {
        layout: "fullscreen",
    },
};

export default meta;
type Story = StoryObj<typeof HomePage>;

export const LoggedOut: Story = {
    args: {
        isLoggedIn: false,
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("button", { name: "Sign Up" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Log In" })).toBeVisible();
        await expect(
            canvas.queryByRole("button", { name: "View Dashboard" }),
        ).not.toBeInTheDocument();
        await userEvent.click(canvas.getByRole("button", { name: "Sign Up" }));
        await expect(canvas.getByText("Sign Up Route")).toBeVisible();
    },
};

export const LoggedIn: Story = {
    args: {
        isLoggedIn: true,
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(
            canvas.queryByRole("button", { name: "Sign Up" }),
        ).not.toBeInTheDocument();
        await expect(
            canvas.queryByRole("button", { name: "Log In" }),
        ).not.toBeInTheDocument();
        await userEvent.click(
            canvas.getByRole("button", { name: "View Dashboard" }),
        );
        await expect(canvas.getByText("Dashboard Route")).toBeVisible();
    },
};
