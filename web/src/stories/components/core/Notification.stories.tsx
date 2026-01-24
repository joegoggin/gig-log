import type { Meta, StoryObj } from "@storybook/react-vite";
import Notification, { NotificationType } from "@/components/core/Notification/Notification";

const meta: Meta<typeof Notification> = {
    title: "Core/Notification",
    component: Notification,
    tags: ["autodocs"],
    argTypes: {
        type: {
            control: { type: "select" },
            options: [
                NotificationType.INFO,
                NotificationType.WARNING,
                NotificationType.SUCCESS,
                NotificationType.ERROR,
            ],
        },
    },
};

export default meta;
type Story = StoryObj<typeof Notification>;

export const Info: Story = {
    args: {
        type: NotificationType.INFO,
        title: "Information",
        message: "This is an informational notification.",
    },
};

export const Warning: Story = {
    args: {
        type: NotificationType.WARNING,
        title: "Warning",
        message: "This is a warning notification.",
    },
};

export const Success: Story = {
    args: {
        type: NotificationType.SUCCESS,
        title: "Success",
        message: "Your changes have been saved successfully.",
    },
};

export const Error: Story = {
    args: {
        type: NotificationType.ERROR,
        title: "Error",
        message: "Something went wrong. Please try again.",
    },
};
