/**
 * Storybook interaction tests for Verify-reset-code page behavior.
 *
 * Covered scenarios:
 * - Successful code verification posts expected payload and navigates
 *   to set-password page.
 * - Missing email in route context is blocked by local validation.
 * - API validation errors are shown for reset-code input.
 */
import { expect, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import VerifyResetCodePage from "@/pages/auth/VerifyResetCodePage/VerifyResetCodePage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import {
    createMockApiResponse,
    createValidationAxiosError,
    mockApiPostHandler,
} from "@/test-utils/mockApiClient";

type PostCall = {
    url: string;
    data: unknown;
};

const meta: Meta<typeof VerifyResetCodePage> = {
    title: "Pages/VerifyResetCodePage",
    component: VerifyResetCodePage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
    },
};

export default meta;
type Story = StoryObj<typeof VerifyResetCodePage>;

export const VerifiesCodeAndNavigates: Story = {
    args: {
        email: "user@example.com",
    },
    play: async ({ canvasElement }) => {
        const postCalls: Array<PostCall> = [];
        const restorePost = mockApiPostHandler(async (url, data) => {
            postCalls.push({ url, data });
            return createMockApiResponse({ message: "Code verified" });
        });

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("Enter reset code"), "RESET123");
            await userEvent.click(canvas.getByRole("button", { name: "Verify Code" }));

            await waitFor(() => {
                expect(postCalls).toHaveLength(1);
            });
            await expect(postCalls[0]).toEqual({
                url: "/auth/verify-forgot-password",
                data: {
                    email: "user@example.com",
                    auth_code: "RESET123",
                },
            });
            await expect(canvas.getByText("Set Password Route")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};

export const MissingEmailShowsValidationError: Story = {
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await userEvent.click(canvas.getByRole("button", { name: "Verify Code" }));
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
                { field: "auth_code", message: "Reset code is invalid" },
            ]);
        });

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("Enter reset code"), "BAD");
            await userEvent.click(canvas.getByRole("button", { name: "Verify Code" }));
            await expect(canvas.getByText("Reset code is invalid")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};
