import { expect, within } from "storybook/test";
import {
    Outlet,
    RouterProvider,
    createMemoryHistory,
    createRootRoute,
    createRoute,
    createRouter,
} from "@tanstack/react-router";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { ReactNode } from "react";
import PrivateRoute from "@/components/auth/PrivateRoute/PrivateRoute";
import { AuthContext } from "@/contexts/AuthContext";

type PrivateRouteStoryProps = {
    isLoggedIn: boolean;
    isLoading: boolean;
    children: ReactNode;
    redirectTo?: string;
    loadingLabel?: string;
};

const createAuthValue = (isLoggedIn: boolean, isLoading: boolean) =>
    ({
        user: isLoggedIn
            ? {
                  id: "user-1",
                  first_name: "Demo",
                  last_name: "User",
                  email: "demo@example.com",
                  email_confirmed: true,
                  created_at: "2024-01-01",
                  updated_at: "2024-01-01",
              }
            : null,
        isLoggedIn,
        isLoading,
        refreshUser: async () => {},
        setUser: () => {},
    }) satisfies NonNullable<React.ContextType<typeof AuthContext>>;

const PrivateRouteStory = ({
    isLoggedIn,
    isLoading,
    children,
    redirectTo,
    loadingLabel,
}: PrivateRouteStoryProps) => {
    const authValue = createAuthValue(isLoggedIn, isLoading);
    const rootRoute = createRootRoute({
        component: () => <Outlet />,
    });
    const privateRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: "/",
        component: () => (
            <AuthContext.Provider value={authValue}>
                <PrivateRoute
                    redirectTo={redirectTo}
                    loadingLabel={loadingLabel}
                >
                    {children}
                </PrivateRoute>
            </AuthContext.Provider>
        ),
    });
    const loginRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: "/auth/log-in",
        component: () => <div>Log In</div>,
    });
    const routeTree = rootRoute.addChildren([privateRoute, loginRoute]);
    const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ["/"] }),
        context: {},
    });

    return <RouterProvider router={router} />;
};

const meta: Meta<typeof PrivateRouteStory> = {
    title: "Auth/PrivateRoute",
    component: PrivateRouteStory,
    tags: ["autodocs"],
    parameters: {
        layout: "fullscreen",
    },
    argTypes: {
        isLoggedIn: {
            control: { type: "boolean" },
        },
        isLoading: {
            control: { type: "boolean" },
        },
        redirectTo: {
            control: { type: "text" },
        },
        loadingLabel: {
            control: { type: "text" },
        },
    },
};

export default meta;
type Story = StoryObj<typeof PrivateRouteStory>;

export const Authenticated: Story = {
    args: {
        isLoggedIn: true,
        isLoading: false,
        children: <div>Protected content</div>,
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("Protected content")).toBeVisible();
    },
};

export const Loading: Story = {
    args: {
        isLoggedIn: false,
        isLoading: true,
        loadingLabel: "Checking session",
        children: <div>Protected content</div>,
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("Checking session")).toBeVisible();
    },
};

export const Redirecting: Story = {
    args: {
        isLoggedIn: false,
        isLoading: false,
        children: <div>Protected content</div>,
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("Log In")).toBeVisible();
        await expect(canvas.queryByText("Protected content")).toBeNull();
    },
};
