/**
 * Storybook interaction tests for Forgot-password page behavior.
 *
 * Covered scenarios:
 * - Successful reset request sends expected payload, shows success notification,
 *   and routes to reset-code verification.
 * - Validation errors from API are displayed in the email field.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import ForgotPasswordPage from "@/pages/auth/ForgotPasswordPage/ForgotPasswordPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
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

const meta: Meta<typeof ForgotPasswordPage> = {
    title: "Pages/ForgotPasswordPage",
    component: ForgotPasswordPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
    },
};

export default meta;
type Story = StoryObj<typeof ForgotPasswordPage>;

export const SendsResetCodeAndNavigates: Story = {
    parameters: {
        storyTest: {
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const postCalls: Array<PostCall> = [];
        const restorePost = mockApiPostHandler(async (url, data) => {
            postCalls.push({ url, data });
            return createMockApiResponse({ message: "Reset code sent" });
        });

        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("Email"), "demo@example.com");
            await userEvent.click(canvas.getByRole("button", { name: "Reset Password" }));

            await waitFor(() => {
                expect(postCalls).toHaveLength(1);
            });
            await expect(postCalls[0]).toEqual({
                url: "/auth/forgot-password",
                data: {
                    email: "demo@example.com",
                },
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Reset Code Sent",
                message: "Please check your email for the reset code.",
            });
            await expect(canvas.getByText("Verify Reset Code Route")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};

export const ShowsValidationErrorsFromApi: Story = {
    play: async ({ canvasElement }) => {
        const restorePost = mockApiPostHandler(async () => {
            throw createValidationAxiosError([
                { field: "email", message: "Email is invalid" },
            ]);
        });

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("Email"), "invalid");
            await userEvent.click(canvas.getByRole("button", { name: "Reset Password" }));
            await expect(canvas.getByText("Email is invalid")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};
