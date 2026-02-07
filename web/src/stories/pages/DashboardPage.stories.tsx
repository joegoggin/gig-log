/**
 * Storybook interaction tests for Dashboard page behavior.
 *
 * Covered scenarios:
 * - Default render exposes dashboard heading and log-out action.
 * - Successful logout calls API, clears auth user, and redirects to log in.
 * - Failed logout surfaces an error notification and does not clear auth state.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { NotificationType } from "@/components/core/Notification/Notification";
import DashboardPage from "@/pages/DashboardPage/DashboardPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import {
    createAxiosErrorResponse,
    createMockApiResponse,
    mockApiPostHandler,
} from "@/test-utils/mockApiClient";

const setUserSpy = fn();
const addNotificationSpy = fn();

type PostCall = {
    url: string;
    data: unknown;
};

const meta: Meta<typeof DashboardPage> = {
    title: "Pages/DashboardPage",
    component: DashboardPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
    },
};

export default meta;
type Story = StoryObj<typeof DashboardPage>;

export const Default: Story = {
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("heading", { name: "Dashboard Page" })).toBeVisible();
        await expect(canvas.getByRole("button", { name: "Log Out" })).toBeVisible();
    },
};

export const LogsOutAndRedirects: Story = {
    parameters: {
        storyTest: {
            auth: {
                setUser: setUserSpy,
            },
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const postCalls: PostCall[] = [];
        const restorePost = mockApiPostHandler(async (url, data) => {
            postCalls.push({ url, data });
            return createMockApiResponse({ message: "Logged out" });
        });

        setUserSpy.mockClear();
        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.click(canvas.getByRole("button", { name: "Log Out" }));

            await waitFor(() => {
                expect(postCalls).toHaveLength(1);
            });
            await expect(postCalls[0]).toEqual({
                url: "/auth/log-out",
                data: undefined,
            });
            await expect(setUserSpy).toHaveBeenCalledWith(null);
            await expect(canvas.getByText("Log In Route")).toBeVisible();
            await expect(addNotificationSpy).not.toHaveBeenCalled();
        } finally {
            restorePost();
        }
    },
};

export const ShowsErrorNotificationWhenLogoutFails: Story = {
    parameters: {
        storyTest: {
            auth: {
                setUser: setUserSpy,
            },
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const restorePost = mockApiPostHandler(async () => {
            throw createAxiosErrorResponse(
                { error: "Session expired" },
                401,
                "Unauthorized",
            );
        });

        setUserSpy.mockClear();
        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.click(canvas.getByRole("button", { name: "Log Out" }));

            await waitFor(() => {
                expect(addNotificationSpy).toHaveBeenCalledWith({
                    type: NotificationType.ERROR,
                    title: "Log Out Failed",
                    message: "Session expired",
                });
            });
            await expect(canvas.getByText("Log Out Failed")).toBeVisible();
            await expect(canvas.getByText("Session expired")).toBeVisible();
            await expect(setUserSpy).not.toHaveBeenCalled();
        } finally {
            restorePost();
        }
    },
};
