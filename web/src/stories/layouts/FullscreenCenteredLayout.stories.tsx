import type { Meta, StoryObj } from "@storybook/react-vite";
import { NotificationProvider } from "@/contexts/NotificationContext";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";

const meta: Meta<typeof FullscreenCenteredLayout> = {
    title: "Layouts/FullscreenCenteredLayout",
    component: FullscreenCenteredLayout,
    tags: ["autodocs"],
    parameters: {
        layout: "fullscreen",
    },
    decorators: [
        (Story) => (
            <NotificationProvider>
                <Story />
            </NotificationProvider>
        ),
    ],
};

export default meta;
type Story = StoryObj<typeof FullscreenCenteredLayout>;

export const Default: Story = {
    args: {
        children: (
            <div style={{ textAlign: "center" }}>
                <h1>Centered Content</h1>
                <p>This content is centered both horizontally and vertically.</p>
            </div>
        ),
    },
};

export const LoginExample: Story = {
    args: {
        className: "login-page",
        children: (
            <div style={{ textAlign: "center", maxWidth: "400px" }}>
                <h1>Login</h1>
                <p>Enter your credentials to continue.</p>
                <form style={{ marginTop: "1rem" }}>
                    <input
                        type="email"
                        placeholder="Email"
                        style={{
                            display: "block",
                            width: "100%",
                            marginBottom: "0.5rem",
                            padding: "0.5rem",
                        }}
                    />
                    <input
                        type="password"
                        placeholder="Password"
                        style={{
                            display: "block",
                            width: "100%",
                            marginBottom: "1rem",
                            padding: "0.5rem",
                        }}
                    />
                    <button type="submit" style={{ padding: "0.5rem 1rem" }}>
                        Sign In
                    </button>
                </form>
            </div>
        ),
    },
};
