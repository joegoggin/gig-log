/**
 * Storybook interaction tests for Sign-up page behavior.
 *
 * Covered scenarios:
 * - Successful submission sends expected payload, shows success notification,
 *   and navigates to confirm-email flow.
 * - Validation responses from the API are rendered at field level.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { NotificationType } from "@/components/core/Notification/Notification";
import SignUpPage from "@/pages/auth/SignUpPage/SignUpPage";
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

const meta: Meta<typeof SignUpPage> = {
    title: "Pages/SignUpPage",
    component: SignUpPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
    },
};

export default meta;
type Story = StoryObj<typeof SignUpPage>;

export const SubmitsAndNavigatesToConfirmEmail: Story = {
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
            return createMockApiResponse({ message: "Account created" });
        });

        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("First Name"), "John");
            await userEvent.type(canvas.getByPlaceholderText("Last Name"), "Doe");
            await userEvent.type(canvas.getByPlaceholderText("Email"), "john@example.com");
            await userEvent.type(canvas.getByPlaceholderText("Password"), "password123");
            await userEvent.type(
                canvas.getByPlaceholderText("Confirm Password"),
                "password123",
            );

            await userEvent.click(canvas.getByRole("button", { name: "Sign Up" }));

            await waitFor(() => {
                expect(postCalls).toHaveLength(1);
            });
            await expect(postCalls[0]).toEqual({
                url: "/auth/sign-up",
                data: {
                    first_name: "John",
                    last_name: "Doe",
                    email: "john@example.com",
                    password: "password123",
                    confirm: "password123",
                },
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Account Created",
                message: "Please check your email to confirm your account.",
            });
            await expect(canvas.getByText("Confirm Email Route")).toBeVisible();
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
            await userEvent.type(canvas.getByPlaceholderText("First Name"), "John");
            await userEvent.type(canvas.getByPlaceholderText("Last Name"), "Doe");
            await userEvent.type(canvas.getByPlaceholderText("Email"), "invalid-email");
            await userEvent.type(canvas.getByPlaceholderText("Password"), "password123");
            await userEvent.type(
                canvas.getByPlaceholderText("Confirm Password"),
                "password123",
            );
            await userEvent.click(canvas.getByRole("button", { name: "Sign Up" }));

            await expect(canvas.getByText("Email is invalid")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};
