import { expect, userEvent, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { NotificationProvider } from "@/contexts/NotificationContext";
import HomePage from "@/pages/HomePage/HomePage";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof HomePage> = {
    title: "Pages/HomePage",
    component: HomePage,
    tags: ["autodocs"],
    decorators: [
        withMemoryRouter,
        (Story) => (
            <NotificationProvider>
                <Story />
            </NotificationProvider>
        ),
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
        await userEvent.click(
            canvas.getByRole("button", { name: "View Dashboard" }),
        );
        await expect(canvas.getByText("Dashboard Route")).toBeVisible();
    },
};
