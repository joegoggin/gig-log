/**
 * Storybook interaction tests for Settings account-security flows.
 *
 * Covered scenarios:
 * - Password change submits expected payload and renders validation errors.
 * - Email-change request submits normalized payload and handles validation failures.
 * - Email-change confirmation submits expected payload, triggers auth refresh,
 *   and renders validation errors.
 *
 * Regression focus:
 * - Prevents API contract drift for settings form payloads.
 * - Ensures success/error feedback remains visible to users.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import SettingsPage from "@/pages/SettingsPage/SettingsPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import {
    createMockApiResponse,
    createValidationAxiosError,
    mockApiPostHandler,
} from "@/test-utils/mockApiClient";

const addNotificationSpy = fn();
const refreshUserSpy = fn(async () => {});

type PostCall = {
    url: string;
    data: unknown;
};

const meta: Meta<typeof SettingsPage> = {
    title: "Pages/SettingsPage",
    component: SettingsPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/settings",
                initialEntries: ["/settings"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof SettingsPage>;

export const ChangesPasswordAndShowsSuccessNotification: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/settings",
                initialEntries: ["/settings"],
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

export const RequestsEmailChangeCodeAndShowsConfirmStep: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/settings",
                initialEntries: ["/settings"],
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
                createMockApiResponse({
                    message: "If this email is available, a confirmation code has been sent.",
                }),
            );
        });

        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("New Email"), "NEW@EMAIL.COM");
            await userEvent.click(
                canvas.getByRole("button", { name: "Send Confirmation Code" }),
            );

            await waitFor(() => {
                expect(postCalls).toHaveLength(1);
            });
            await expect(postCalls[0]).toEqual({
                url: "/auth/request-email-change",
                data: {
                    new_email: "new@email.com",
                },
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.INFO,
                title: "Confirmation Code Sent",
                message: "If this email is available, a confirmation code has been sent.",
            });
            await expect(
                canvas.getByText("Enter the code sent to new@email.com."),
            ).toBeVisible();
            await expect(
                canvas.getByRole("button", { name: "Resend Confirmation Code" }),
            ).toBeVisible();
        } finally {
            restorePost();
        }
    },
};

export const ShowsEmailRequestValidationErrors: Story = {
    play: async ({ canvasElement }) => {
        const restorePost = mockApiPostHandler(() =>
            Promise.reject(
                createValidationAxiosError([
                    {
                        field: "new_email",
                        message: "Email is invalid",
                    },
                ]),
            ),
        );

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("New Email"), "not-an-email");
            await userEvent.click(
                canvas.getByRole("button", { name: "Send Confirmation Code" }),
            );
            await expect(canvas.getByText("Email is invalid")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};

export const ConfirmsEmailChangeAndRefreshesUser: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/settings",
                initialEntries: ["/settings"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
                refreshUser: refreshUserSpy,
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

            if (url === "/auth/request-email-change") {
                return Promise.resolve(
                    createMockApiResponse({
                        message:
                            "If this email is available, a confirmation code has been sent.",
                    }),
                );
            }

            return Promise.resolve(
                createMockApiResponse({ message: "Email changed successfully." }),
            );
        });

        addNotificationSpy.mockClear();
        refreshUserSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("New Email"), "next@example.com");
            await userEvent.click(
                canvas.getByRole("button", { name: "Send Confirmation Code" }),
            );

            await userEvent.type(
                canvas.getByPlaceholderText("Email Change Code"),
                "CODE123",
            );
            await userEvent.click(
                canvas.getByRole("button", { name: "Confirm Email Change" }),
            );

            await waitFor(() => {
                expect(postCalls).toHaveLength(2);
            });
            await expect(postCalls[0]).toEqual({
                url: "/auth/request-email-change",
                data: {
                    new_email: "next@example.com",
                },
            });
            await expect(postCalls[1]).toEqual({
                url: "/auth/confirm-email-change",
                data: {
                    new_email: "next@example.com",
                    auth_code: "CODE123",
                },
            });
            await expect(refreshUserSpy).toHaveBeenCalledTimes(1);
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Email Updated",
                message: "Email changed successfully.",
            });
        } finally {
            restorePost();
        }
    },
};

export const ShowsEmailConfirmValidationErrors: Story = {
    play: async ({ canvasElement }) => {
        const postCalls: Array<PostCall> = [];
        const restorePost = mockApiPostHandler((url, data) => {
            postCalls.push({ url, data });

            if (url === "/auth/request-email-change") {
                return Promise.resolve(
                    createMockApiResponse({
                        message:
                            "If this email is available, a confirmation code has been sent.",
                    }),
                );
            }

            return Promise.reject(
                createValidationAxiosError([
                    {
                        field: "auth_code",
                        message: "Auth code is required",
                    },
                ]),
            );
        });

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("New Email"), "next@example.com");
            await userEvent.click(
                canvas.getByRole("button", { name: "Send Confirmation Code" }),
            );

            await userEvent.click(
                canvas.getByRole("button", { name: "Confirm Email Change" }),
            );

            await waitFor(() => {
                expect(postCalls).toHaveLength(2);
            });
            await expect(canvas.getByText("Auth code is required")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};
