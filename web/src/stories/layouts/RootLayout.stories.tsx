import { useEffect } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { NotificationType } from "@/components/core/Notification/Notification";
import { NotificationProvider, useNotification } from "@/contexts/NotificationContext";
import RootLayout from "@/layouts/RootLayout/RootLayout";

const meta: Meta<typeof RootLayout> = {
    title: "Layouts/RootLayout",
    component: RootLayout,
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
type Story = StoryObj<typeof RootLayout>;

export const Default: Story = {
    args: {
        children: (
            <div style={{ padding: "2rem" }}>
                <h1>Page Content</h1>
                <p>This is sample content inside the RootLayout.</p>
            </div>
        ),
    },
};

export const WithClassName: Story = {
    args: {
        className: "custom-page",
        children: (
            <div style={{ padding: "2rem" }}>
                <h1>Custom Page</h1>
                <p>This layout has an additional custom class applied.</p>
            </div>
        ),
    },
};

function NotificationTrigger() {
    const { addNotification } = useNotification();

    useEffect(() => {
        addNotification({
            type: NotificationType.SUCCESS,
            title: "Email Confirmed",
            message: "Your email has been confirmed. You can now log in.",
        });
    }, [addNotification]);

    return (
        <div style={{ padding: "2rem" }}>
            <h1>Page with Notification</h1>
            <p>A notification should appear at the top of this layout.</p>
        </div>
    );
}

export const WithNotification: Story = {
    render: () => (
        <RootLayout>
            <NotificationTrigger />
        </RootLayout>
    ),
};
