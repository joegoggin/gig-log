/**
 * Storybook interaction tests for MainLayout navigation behavior.
 *
 * Covered scenarios:
 * - Active route state is shown for the current destination.
 * - Menu navigation transitions to selected placeholder routes.
 * - Log-out action clears auth on success and shows notification on failure.
 * - Mobile menu toggle and drawer interactions work as expected.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import MainLayout from "@/layouts/MainLayout/MainLayout";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import {
    createAxiosErrorResponse,
    createMockApiResponse,
    mockApiPostHandler,
} from "@/test-utils/mockApiClient";

const setUserSpy = fn();
const addNotificationSpy = fn();

const meta: Meta<typeof MainLayout> = {
    title: "Layouts/MainLayout",
    component: MainLayout,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/dashboard",
                initialEntries: ["/dashboard"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
                setUser: setUserSpy,
            },
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof MainLayout>;

export const Default: Story = {
    args: {
        children: <h1>Dashboard</h1>,
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        const dashboardButton = canvas.getByRole("button", { name: "Dashboard" });
        const jobsButton = canvas.getByRole("button", { name: "Jobs" });
        await expect(dashboardButton).toBeVisible();
        await expect(dashboardButton).toHaveAttribute("aria-current", "page");
        await expect(jobsButton).not.toHaveAttribute("aria-current");
        await expect(canvas.getByRole("button", { name: "Log Out" })).toBeVisible();
    },
};

export const NavigatesFromSidebar: Story = {
    args: {
        children: <h1>Dashboard</h1>,
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await userEvent.click(canvas.getByRole("button", { name: "Jobs" }));
        await expect(canvas.getByText("Jobs Route")).toBeVisible();
    },
};

export const NavigatesToDashboardFromBrand: Story = {
    args: {
        children: <h1>Dashboard</h1>,
    },
    parameters: {
        storyTest: {
            router: {
                storyPath: "/jobs",
                initialEntries: ["/jobs"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
                setUser: setUserSpy,
            },
            spies: {
                addNotification: addNotificationSpy,
            },
        } satisfies StoryTestParameters["storyTest"],
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await userEvent.click(
            canvas.getByRole("button", { name: "Go to dashboard" }),
        );
        await expect(canvas.getByText("Dashboard Route")).toBeVisible();
    },
};

export const LogsOutOnSuccess: Story = {
    args: {
        children: <h1>Dashboard</h1>,
    },
    parameters: {
        storyTest: {
            router: {
                storyPath: "/dashboard",
                initialEntries: ["/dashboard"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
                setUser: setUserSpy,
            },
            spies: {
                addNotification: addNotificationSpy,
            },
        } satisfies StoryTestParameters["storyTest"],
    },
    play: async ({ canvasElement }) => {
        const restorePost = mockApiPostHandler(async () => {
            return createMockApiResponse({ message: "Logged out" });
        });

        setUserSpy.mockClear();
        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.click(canvas.getByRole("button", { name: "Log Out" }));

            await waitFor(() => {
                expect(setUserSpy).toHaveBeenCalledWith(null);
            });
            await expect(canvas.getByText("Log In Route")).toBeVisible();
            await expect(addNotificationSpy).not.toHaveBeenCalled();
        } finally {
            restorePost();
        }
    },
};

export const ShowsErrorWhenLogoutFails: Story = {
    args: {
        children: <h1>Dashboard</h1>,
    },
    parameters: {
        storyTest: {
            router: {
                storyPath: "/dashboard",
                initialEntries: ["/dashboard"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
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
        } finally {
            restorePost();
        }
    },
};

export const MobileMenuToggleFlow: Story = {
    args: {
        children: <h1>Dashboard</h1>,
    },
    parameters: {
        chromatic: {
            viewports: [390],
        },
        storyTest: {
            router: {
                storyPath: "/dashboard",
                initialEntries: ["/dashboard"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
                setUser: setUserSpy,
            },
            spies: {
                addNotification: addNotificationSpy,
            },
        } satisfies StoryTestParameters["storyTest"],
    },
    play: async ({ canvasElement }) => {
        const rootElement = canvasElement.ownerDocument.documentElement;
        rootElement.setAttribute("data-force-mobile-nav", "true");

        try {
            const canvas = within(canvasElement);

            await userEvent.click(
                canvas.getByRole("button", { name: "Open navigation menu" }),
            );
            await expect(
                canvas.getByRole("button", { name: "Close navigation menu" }),
            ).toBeVisible();

            const drawer = canvasElement.ownerDocument.getElementById(
                "main-layout-mobile-drawer",
            );
            expect(drawer).toBeTruthy();

            const viewportWidth =
                canvasElement.ownerDocument.defaultView?.innerWidth ?? 0;
            const drawerBounds = drawer?.getBoundingClientRect();

            expect(drawerBounds).toBeTruthy();
            if (!drawerBounds) {
                throw new Error("Mobile drawer bounds were not available");
            }

            expect(drawerBounds.width).toBeGreaterThan(0);
            expect(drawerBounds.left).toBeGreaterThanOrEqual(0);
            expect(drawerBounds.right).toBeLessThanOrEqual(viewportWidth + 1);

            await userEvent.click(
                canvas.getByRole("button", { name: "Dismiss navigation overlay" }),
            );
            await expect(
                canvas.getByRole("button", { name: "Open navigation menu" }),
            ).toBeVisible();
        } finally {
            rootElement.removeAttribute("data-force-mobile-nav");
        }
    },
};

export const MobileDrawerNavigation: Story = {
    args: {
        children: <h1>Dashboard</h1>,
    },
    parameters: {
        chromatic: {
            viewports: [390],
        },
        storyTest: {
            router: {
                storyPath: "/dashboard",
                initialEntries: ["/dashboard"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
                setUser: setUserSpy,
            },
            spies: {
                addNotification: addNotificationSpy,
            },
        } satisfies StoryTestParameters["storyTest"],
    },
    play: async ({ canvasElement }) => {
        const rootElement = canvasElement.ownerDocument.documentElement;
        rootElement.setAttribute("data-force-mobile-nav", "true");

        try {
            const canvas = within(canvasElement);
            await userEvent.click(
                canvas.getByRole("button", { name: "Open navigation menu" }),
            );
            await userEvent.click(canvas.getByRole("button", { name: "Jobs" }));
            await expect(canvas.getByText("Jobs Route")).toBeVisible();
        } finally {
            rootElement.removeAttribute("data-force-mobile-nav");
        }
    },
};
