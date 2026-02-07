/**
 * Unit tests for Sign-up page callback and side-effect behavior.
 *
 * Covered scenarios:
 * - Successful sign-up posts expected payload, dispatches success notification,
 *   and navigates to confirm-email route with search params.
 * - Validation error responses are rendered and block navigation.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import SignUpPage from "./SignUpPage";
import type * as TanStackRouter from "@tanstack/react-router";
import { NotificationType } from "@/components/core/Notification/Notification";
import { NotificationContext } from "@/contexts/NotificationContext";
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

type NotificationCall = {
    type: NotificationType;
    title: string;
    message: string;
};

const renderPage = (addNotification: (notification: NotificationCall) => void) => {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: { retry: false },
            mutations: { retry: false },
        },
    });

    render(
        <QueryClientProvider client={queryClient}>
            <NotificationContext.Provider
                value={{
                    notifications: [],
                    addNotification,
                    removeNotification: () => {},
                }}
            >
                <SignUpPage />
            </NotificationContext.Provider>
        </QueryClientProvider>,
    );
};

describe("SignUpPage", () => {
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

    it("submits user data, notifies success, and navigates to confirm email", async () => {
        const addNotification = vi.fn();
        const postCalls: Array<{ url: string; data: unknown }> = [];

        restorePost = mockApiPostHandler(async (url, data) => {
            postCalls.push({ url, data });
            return createMockApiResponse({ message: "Account created" });
        });

        renderPage(addNotification);

        fireEvent.change(screen.getByPlaceholderText("First Name"), {
            target: { value: "John" },
        });
        fireEvent.change(screen.getByPlaceholderText("Last Name"), {
            target: { value: "Doe" },
        });
        fireEvent.change(screen.getByPlaceholderText("Email"), {
            target: { value: "john@example.com" },
        });
        fireEvent.change(screen.getByPlaceholderText("Password"), {
            target: { value: "password123" },
        });
        fireEvent.change(screen.getByPlaceholderText("Confirm Password"), {
            target: { value: "password123" },
        });

        fireEvent.click(screen.getByRole("button", { name: "Sign Up" }));

        await waitFor(() => {
            expect(postCalls).toHaveLength(1);
        });
        expect(postCalls[0]).toEqual({
            url: "/auth/sign-up",
            data: {
                first_name: "John",
                last_name: "Doe",
                email: "john@example.com",
                password: "password123",
                confirm: "password123",
            },
        });

        expect(addNotification).toHaveBeenCalledWith({
            type: NotificationType.SUCCESS,
            title: "Account Created",
            message: "Please check your email to confirm your account.",
        });
        expect(navigateMock).toHaveBeenCalledWith({
            to: "/auth/confirm-email",
            search: {
                email: "john@example.com",
            },
        });
    });

    it("shows field validation errors from API", async () => {
        const addNotification = vi.fn();

        restorePost = mockApiPostHandler(async () => {
            throw createValidationAxiosError([
                { field: "email", message: "Email is invalid" },
            ]);
        });

        renderPage(addNotification);

        fireEvent.change(screen.getByPlaceholderText("Email"), {
            target: { value: "invalid-email" },
        });
        fireEvent.click(screen.getByRole("button", { name: "Sign Up" }));

        await expect(screen.findByText("Email is invalid")).resolves.toBeTruthy();
        expect(addNotification).not.toHaveBeenCalled();
        expect(navigateMock).not.toHaveBeenCalled();
    });
});
