/**
 * Unit tests for Log-in page callback and side-effect behavior.
 *
 * Covered scenarios:
 * - Successful log in posts credentials, refreshes auth context,
 *   and navigates to dashboard.
 * - Validation error responses are rendered and block navigation.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import LogInPage from "./LogInPage";
import type * as TanStackRouter from "@tanstack/react-router";
import { AuthContext } from "@/contexts/AuthContext";
import { NotificationProvider } from "@/contexts/NotificationContext";
import {
    createMockApiResponse,
    createValidationAxiosError,
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

const setUser = vi.fn();

const renderPage = (refreshUser: () => Promise<void>) => {
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
                    user: null,
                    isLoggedIn: false,
                    isLoading: false,
                    refreshUser,
                    setUser,
                }}
            >
                <NotificationProvider>
                    <LogInPage />
                </NotificationProvider>
            </AuthContext.Provider>
        </QueryClientProvider>,
    );
};

describe("LogInPage", () => {
    beforeEach(() => {
        navigateMock.mockReset();
        setUser.mockReset();
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

    it("refreshes auth and navigates to dashboard after successful log in", async () => {
        const refreshUser = vi.fn(async () => {});
        const postCalls: Array<{ url: string; data: unknown }> = [];

        restorePost = mockApiPostHandler(async (url, data) => {
            postCalls.push({ url, data });
            return createMockApiResponse({
                message: "Logged in",
                user_id: "user-1",
            });
        });

        renderPage(refreshUser);

        fireEvent.change(screen.getByPlaceholderText("Email"), {
            target: { value: "demo@example.com" },
        });
        fireEvent.change(screen.getByPlaceholderText("Password"), {
            target: { value: "password123" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Log In" }));

        await waitFor(() => {
            expect(postCalls).toHaveLength(1);
        });
        expect(postCalls[0]).toEqual({
            url: "/auth/log-in",
            data: {
                email: "demo@example.com",
                password: "password123",
            },
        });

        await waitFor(() => {
            expect(refreshUser).toHaveBeenCalledTimes(1);
        });
        expect(navigateMock).toHaveBeenCalledWith({ to: "/dashboard" });
    });

    it("renders field errors from validation response", async () => {
        const refreshUser = vi.fn(async () => {});

        restorePost = mockApiPostHandler(async () => {
            throw createValidationAxiosError([
                {
                    field: "password",
                    message: "Password is required",
                },
            ]);
        });

        renderPage(refreshUser);

        fireEvent.change(screen.getByPlaceholderText("Email"), {
            target: { value: "demo@example.com" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Log In" }));

        await expect(
            screen.findByText("Password is required"),
        ).resolves.toBeTruthy();
        expect(navigateMock).not.toHaveBeenCalled();
    });
});
