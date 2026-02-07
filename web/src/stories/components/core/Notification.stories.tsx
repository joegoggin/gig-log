import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import Notification, { NotificationType } from "@/components/core/Notification/Notification";
import styles from "@/components/core/Notification/Notification.module.scss";

const iconTestIds = [
    "notification-icon-info",
    "notification-icon-warning",
    "notification-icon-success",
    "notification-icon-error",
] as const;

const notificationStylesByType: Record<
    NotificationType,
    {
        backgroundColor: string;
        iconTestId: (typeof iconTestIds)[number];
        variantClass: string;
    }
> = {
    [NotificationType.INFO]: {
        backgroundColor: "rgb(122, 162, 247)",
        iconTestId: "notification-icon-info",
        variantClass: styles["notification--info"],
    },
    [NotificationType.WARNING]: {
        backgroundColor: "rgb(224, 175, 104)",
        iconTestId: "notification-icon-warning",
        variantClass: styles["notification--warning"],
    },
    [NotificationType.SUCCESS]: {
        backgroundColor: "rgb(158, 206, 106)",
        iconTestId: "notification-icon-success",
        variantClass: styles["notification--success"],
    },
    [NotificationType.ERROR]: {
        backgroundColor: "rgb(247, 118, 142)",
        iconTestId: "notification-icon-error",
        variantClass: styles["notification--error"],
    },
};

const assertNotificationVisuals = (
    canvas: ReturnType<typeof within>,
    type: NotificationType,
) => {
    const expectations = notificationStylesByType[type];
    const root = canvas.getByTestId("notification-root");

    expect(root).toHaveClass(styles["notification"]);
    expect(root).toHaveClass(expectations.variantClass);
    expect(getComputedStyle(root).backgroundColor).toBe(
        expectations.backgroundColor,
    );

    for (const iconTestId of iconTestIds) {
        if (iconTestId === expectations.iconTestId) {
            expect(canvas.getByTestId(iconTestId)).toBeInTheDocument();
        } else {
            expect(canvas.queryByTestId(iconTestId)).not.toBeInTheDocument();
        }
    }
};

const meta: Meta<typeof Notification> = {
    title: "Core/Notification",
    component: Notification,
    tags: ["autodocs"],
    parameters: {
        a11y: {
            test: "error",
        },
    },
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
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("Information")).toBeInTheDocument();
        await expect(
            canvas.getByText("This is an informational notification."),
        ).toBeInTheDocument();
        assertNotificationVisuals(canvas, NotificationType.INFO);
    },
};

export const Warning: Story = {
    args: {
        type: NotificationType.WARNING,
        title: "Warning",
        message: "This is a warning notification.",
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("Warning")).toBeInTheDocument();
        assertNotificationVisuals(canvas, NotificationType.WARNING);
    },
};

export const Success: Story = {
    args: {
        type: NotificationType.SUCCESS,
        title: "Success",
        message: "Your changes have been saved successfully.",
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("Success")).toBeInTheDocument();
        assertNotificationVisuals(canvas, NotificationType.SUCCESS);
    },
};

export const Error: Story = {
    args: {
        type: NotificationType.ERROR,
        title: "Error",
        message: "Something went wrong. Please try again.",
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("Error")).toBeInTheDocument();
        assertNotificationVisuals(canvas, NotificationType.ERROR);
    },
};

export const Dismissible: Story = {
    args: {
        type: NotificationType.INFO,
        title: "Dismiss me",
        message: "Close this notification to run interaction assertions.",
        onClose: fn(),
    },
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement);
        assertNotificationVisuals(canvas, NotificationType.INFO);
        await userEvent.click(
            canvas.getByRole("button", { name: "Close notification" }),
        );
        await waitFor(() => {
            expect(canvas.queryByText("Dismiss me")).not.toBeInTheDocument();
        });
        await expect(args.onClose).toHaveBeenCalledTimes(1);
    },
};
