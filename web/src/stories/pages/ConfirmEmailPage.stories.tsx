/**
 * Storybook interaction tests for Confirm-email page behavior.
 *
 * Covered scenarios:
 * - Successful confirmation sends expected payload, shows success notification,
 *   and navigates to log-in page.
 * - Missing email in route context is blocked by local validation.
 * - Validation errors from API are rendered in the auth-code field.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import ConfirmEmailPage from "@/pages/auth/ConfirmEmailPage/ConfirmEmail";
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

const meta: Meta<typeof ConfirmEmailPage> = {
    title: "Pages/ConfirmEmailPage",
    component: ConfirmEmailPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
    },
};

export default meta;
type Story = StoryObj<typeof ConfirmEmailPage>;

export const ConfirmsEmailAndNavigates: Story = {
    args: {
        email: "user@example.com",
    },
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
            return createMockApiResponse({ message: "Email confirmed" });
        });

        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.type(
                canvas.getByPlaceholderText("Enter confirmation code"),
                "ABC123",
            );
            await userEvent.click(canvas.getByRole("button", { name: "Confirm Email" }));

            await waitFor(() => {
                expect(postCalls).toHaveLength(1);
            });
            await expect(postCalls[0]).toEqual({
                url: "/auth/confirm-email",
                data: {
                    email: "user@example.com",
                    auth_code: "ABC123",
                },
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Email Confirmed",
                message: "Your email has been confirmed. You can now log in.",
            });
            await expect(canvas.getByText("Log In Route")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};

export const MissingEmailShowsValidationError: Story = {
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await userEvent.click(canvas.getByRole("button", { name: "Confirm Email" }));
        await expect(canvas.getByText("Email is required")).toBeVisible();
    },
};

export const ShowsValidationErrorsFromApi: Story = {
    args: {
        email: "user@example.com",
    },
    play: async ({ canvasElement }) => {
        const restorePost = mockApiPostHandler(async () => {
            throw createValidationAxiosError([
                { field: "auth_code", message: "Code is invalid" },
            ]);
        });

        try {
            const canvas = within(canvasElement);
            await userEvent.type(
                canvas.getByPlaceholderText("Enter confirmation code"),
                "BADCODE",
            );
            await userEvent.click(canvas.getByRole("button", { name: "Confirm Email" }));
            await expect(canvas.getByText("Code is invalid")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};
