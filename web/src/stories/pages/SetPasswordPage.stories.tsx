/**
 * Storybook interaction tests for Set-password page behavior.
 *
 * Covered scenarios:
 * - Successful reset sends expected payload, shows success notification,
 *   and routes to log-in page.
 * - API validation errors are shown for password confirmation flow.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { NotificationType } from "@/components/core/Notification/Notification";
import SetPasswordPage from "@/pages/auth/SetPassword/SetPasswordPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import {
    createMockApiResponse,
    createValidationAxiosError,
    mockApiPostHandler,
} from "@/test-utils/mockApiClient";

const addNotificationSpy = fn();

type PostCall = {
    url: string;
    data: unknown;
};

const meta: Meta<typeof SetPasswordPage> = {
    title: "Pages/SetPasswordPage",
    component: SetPasswordPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
    },
};

export default meta;
type Story = StoryObj<typeof SetPasswordPage>;

export const ResetsPasswordAndNavigates: Story = {
    parameters: {
        storyTest: {
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const postCalls: PostCall[] = [];
        const restorePost = mockApiPostHandler(async (url, data) => {
            postCalls.push({ url, data });
            return createMockApiResponse({ message: "Password reset" });
        });

        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("Password"), "newpassword123");
            await userEvent.type(
                canvas.getByPlaceholderText("Confirm Password"),
                "newpassword123",
            );
            await userEvent.click(canvas.getByRole("button", { name: "Set Password" }));

            await waitFor(() => {
                expect(postCalls).toHaveLength(1);
            });
            await expect(postCalls[0]).toEqual({
                url: "/auth/set-password",
                data: {
                    password: "newpassword123",
                    confirm: "newpassword123",
                },
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Password Reset",
                message: "Your password has been reset successfully.",
            });
            await expect(canvas.getByText("Log In Route")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};

export const ShowsValidationErrorsFromApi: Story = {
    play: async ({ canvasElement }) => {
        const restorePost = mockApiPostHandler(async () => {
            throw createValidationAxiosError([
                { field: "confirm", message: "Passwords do not match" },
            ]);
        });

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("Password"), "password123");
            await userEvent.type(canvas.getByPlaceholderText("Confirm Password"), "password321");
            await userEvent.click(canvas.getByRole("button", { name: "Set Password" }));
            await expect(canvas.getByText("Passwords do not match")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};
