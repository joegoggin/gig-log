/**
 * Unit tests for Dashboard page callback and side-effect behavior.
 *
 * Covered scenarios:
 * - Successful logout posts to API, clears auth user, and navigates to log in.
 * - Failed logout dispatches an error notification and preserves auth state.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import DashboardPage from "./DashboardPage";
import type * as TanStackRouter from "@tanstack/react-router";
import { NotificationType } from "@/components/core/Notification/Notification";
import { AuthContext } from "@/contexts/AuthContext";
import { NotificationContext } from "@/contexts/NotificationContext";
import {
    createAxiosErrorResponse,
    createMockApiResponse,
    mockApiPostHandler,
} from "@/test-utils/mockApiClient";

const navigateMock = vi.fn();

vi.mock("@tanstack/react-router", async () => {
    const actual = await vi.importActual<typeof TanStackRouter>(
        "@tanstack/react-router",
    );

    return {
        ...actual,
        useNavigate: () => navigateMock,
    };
});

let restorePost: (() => void) | undefined;

const refreshUser = vi.fn(async () => {});

type NotificationCall = {
    type: NotificationType;
    title: string;
    message: string;
};

const renderPage = (setUser: (user: unknown) => void, addNotification: (notification: NotificationCall) => void) => {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: { retry: false },
            mutations: { retry: false },
        },
    });

    render(
        <QueryClientProvider client={queryClient}>
            <AuthContext.Provider
                value={{
                    user: {
                        id: "user-1",
                        first_name: "Demo",
                        last_name: "User",
                        email: "demo@example.com",
                        email_confirmed: true,
                        created_at: "2024-01-01",
                        updated_at: "2024-01-01",
                    },
                    isLoggedIn: true,
                    isLoading: false,
                    refreshUser,
                    setUser,
                }}
            >
                <NotificationContext.Provider
                    value={{
                        notifications: [],
                        addNotification,
                        removeNotification: () => {},
                    }}
                >
                    <DashboardPage />
                </NotificationContext.Provider>
            </AuthContext.Provider>
        </QueryClientProvider>,
    );
};

describe("DashboardPage", () => {
    beforeEach(() => {
        navigateMock.mockReset();
        Object.defineProperty(window, "matchMedia", {
            writable: true,
            value: vi.fn().mockImplementation((query: string) => ({
                matches: query.includes("dark"),
                media: query,
                onchange: null,
                addListener: vi.fn(),
                removeListener: vi.fn(),
                addEventListener: vi.fn(),
                removeEventListener: vi.fn(),
                dispatchEvent: vi.fn(),
            })),
        });
    });

    afterEach(() => {
        restorePost?.();
        restorePost = undefined;
    });

    it("clears auth user and navigates to log in when logout succeeds", async () => {
        const setUser = vi.fn();
        const addNotification = vi.fn();
        const postCalls: Array<{ url: string; data: unknown }> = [];

        restorePost = mockApiPostHandler(async (url, data) => {
            postCalls.push({ url, data });
            return createMockApiResponse({ message: "Logged out" });
        });

        renderPage(setUser, addNotification);

        fireEvent.click(screen.getByRole("button", { name: "Log Out" }));

        await waitFor(() => {
            expect(postCalls).toHaveLength(1);
        });
        expect(postCalls[0]).toEqual({
            url: "/auth/log-out",
            data: undefined,
        });

        await waitFor(() => {
            expect(setUser).toHaveBeenCalledWith(null);
        });
        expect(navigateMock).toHaveBeenCalledWith({ to: "/auth/log-in" });
        expect(addNotification).not.toHaveBeenCalled();
    });

    it("shows a notification when logout fails", async () => {
        const setUser = vi.fn();
        const addNotification = vi.fn();

        restorePost = mockApiPostHandler(async () => {
            throw createAxiosErrorResponse(
                { error: "Session expired" },
                401,
                "Unauthorized",
            );
        });

        renderPage(setUser, addNotification);

        fireEvent.click(screen.getByRole("button", { name: "Log Out" }));

        await waitFor(() => {
            expect(addNotification).toHaveBeenCalledWith({
                type: NotificationType.ERROR,
                title: "Log Out Failed",
                message: "Session expired",
            });
        });
        expect(setUser).not.toHaveBeenCalled();
        expect(navigateMock).not.toHaveBeenCalled();
    });
});
