/**
 * Unit tests for Dashboard page static content.
 *
 * Covered scenarios:
 * - Renders the dashboard heading.
 * - Renders the dashboard guidance copy.
 */
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import DashboardPage from "./DashboardPage";
import type { ReactNode } from "react";
import { AuthContext } from "@/contexts/AuthContext";
import { NotificationContext } from "@/contexts/NotificationContext";

vi.mock("@/layouts/MainLayout/MainLayout", () => ({
    default: ({ children }: { children: ReactNode }) => <div>{children}</div>,
}));

function renderPage() {
    render(
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
                refreshUser: async () => {},
                setUser: () => {},
            }}
        >
            <NotificationContext.Provider
                value={{
                    notifications: [],
                    addNotification: () => {},
                    removeNotification: () => {},
                }}
            >
                <DashboardPage />
            </NotificationContext.Provider>
        </AuthContext.Provider>,
    );
}

describe("DashboardPage", () => {
    it("renders dashboard content", () => {
        renderPage();

        expect(screen.getByRole("heading", { name: "Dashboard" })).toBeTruthy();
        expect(
            screen.getByText("Welcome back. Use the sidebar to navigate across the app."),
        ).toBeTruthy();
    });
});
