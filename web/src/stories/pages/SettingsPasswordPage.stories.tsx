/**
 * Storybook interaction tests for password settings behavior.
 *
 * Covered scenarios:
 * - Password change submits expected payload and shows success feedback.
 * - API validation errors are rendered for password fields.
 *
 * These tests prevent regressions in account password update flows.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import SettingsPasswordPage from "@/pages/SettingsPasswordPage/SettingsPasswordPage";
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

const meta: Meta<typeof SettingsPasswordPage> = {
    title: "Pages/SettingsPasswordPage",
    component: SettingsPasswordPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/settings/password",
                initialEntries: ["/settings/password"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof SettingsPasswordPage>;

export const ChangesPasswordAndShowsSuccessNotification: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/settings/password",
                initialEntries: ["/settings/password"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const postCalls: Array<PostCall> = [];
        const restorePost = mockApiPostHandler((url, data) => {
            postCalls.push({ url, data });
            return Promise.resolve(
                createMockApiResponse({ message: "Password changed successfully." }),
            );
        });

        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.type(
                canvas.getByPlaceholderText("Current Password"),
                "password123",
            );
            await userEvent.type(
                canvas.getByPlaceholderText("New Password"),
                "new-password-123",
            );
            await userEvent.type(
                canvas.getByPlaceholderText("Confirm New Password"),
                "new-password-123",
            );
            await userEvent.click(
                canvas.getByRole("button", { name: "Change Password" }),
            );

            await waitFor(() => {
                expect(postCalls).toHaveLength(1);
            });
            await expect(postCalls[0]).toEqual({
                url: "/auth/change-password",
                data: {
                    current_password: "password123",
                    new_password: "new-password-123",
                    confirm: "new-password-123",
                },
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Password Updated",
                message: "Password changed successfully.",
            });
        } finally {
            restorePost();
        }
    },
};

export const ShowsPasswordValidationErrors: Story = {
    play: async ({ canvasElement }) => {
        const restorePost = mockApiPostHandler(() =>
            Promise.reject(
                createValidationAxiosError([
                    {
                        field: "new_password",
                        message: "New password must have at least 8 characters",
                    },
                ]),
            ),
        );

        try {
            const canvas = within(canvasElement);
            await userEvent.type(
                canvas.getByPlaceholderText("Current Password"),
                "password123",
            );
            await userEvent.type(canvas.getByPlaceholderText("New Password"), "short");
            await userEvent.type(
                canvas.getByPlaceholderText("Confirm New Password"),
                "short",
            );
            await userEvent.click(
                canvas.getByRole("button", { name: "Change Password" }),
            );
            await expect(
                canvas.getByText("New password must have at least 8 characters"),
            ).toBeVisible();
        } finally {
            restorePost();
        }
    },
};
