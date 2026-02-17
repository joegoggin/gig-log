/**
 * Storybook interaction tests for Log-in page behavior.
 *
 * Covered scenarios:
 * - Successful submission sends credentials, refreshes auth state,
 *   and navigates to dashboard.
 * - Forgot-password link routes to the reset-request page.
 * - Validation errors returned by API are shown in the form.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import LogInPage from "@/pages/auth/LogInPage/LogInPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import {
    createMockApiResponse,
    createValidationAxiosError,
    mockApiPostHandler,
} from "@/test-utils/mockApiClient";

const refreshUserSpy = fn(async () => {});

type PostCall = {
    url: string;
    data: unknown;
};

const meta: Meta<typeof LogInPage> = {
    title: "Pages/LogInPage",
    component: LogInPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
    },
};

export default meta;
type Story = StoryObj<typeof LogInPage>;

export const SubmitsCredentialsAndNavigates: Story = {
    parameters: {
        storyTest: {
            auth: {
                refreshUser: refreshUserSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const postCalls: Array<PostCall> = [];
        const restorePost = mockApiPostHandler(async (url, data) => {
            postCalls.push({ url, data });
            return createMockApiResponse({
                message: "Logged in",
                user_id: "user-1",
            });
        });

        refreshUserSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("Email"), "demo@example.com");
            await userEvent.type(canvas.getByPlaceholderText("Password"), "password123");
            await userEvent.click(canvas.getByRole("button", { name: "Log In" }));

            await waitFor(() => {
                expect(postCalls).toHaveLength(1);
            });
            await expect(postCalls[0]).toEqual({
                url: "/auth/log-in",
                data: {
                    email: "demo@example.com",
                    password: "password123",
                    remember_me: false,
                },
            });
            await expect(refreshUserSpy).toHaveBeenCalledTimes(1);
            await expect(canvas.getByText("Dashboard Route")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};

export const NavigatesToForgotPassword: Story = {
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await userEvent.click(canvas.getByRole("button", { name: "Forgot Password?" }));
        await expect(canvas.getByText("Forgot Password Route")).toBeVisible();
    },
};

export const ShowsValidationErrorsFromApi: Story = {
    play: async ({ canvasElement }) => {
        const restorePost = mockApiPostHandler(async () => {
            throw createValidationAxiosError([
                { field: "password", message: "Password is required" },
            ]);
        });

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("Email"), "demo@example.com");
            await userEvent.click(canvas.getByRole("button", { name: "Log In" }));
            await expect(canvas.getByText("Password is required")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};
