/**
 * Storybook decorator for in-memory routing during interaction tests.
 *
 * Covered behavior:
 * - Mounts stories at configurable `storyPath`.
 * - Supports configurable `initialEntries` for navigation/redirect assertions.
 * - Provides route stubs for all auth/dashboard paths used by page stories.
 */
import {
    Outlet,
    RouterProvider,
    createMemoryHistory,
    createRootRoute,
    createRoute,
    createRouter,
} from "@tanstack/react-router";
import type { Decorator } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";

type RouteStubDefinition = {
    path: string;
    label: string;
};

const routeStubs: Array<RouteStubDefinition> = [
    { path: "/dashboard", label: "Dashboard Route" },
    { path: "/companies", label: "Companies Route" },
    { path: "/companies/create", label: "Create Company Route" },
    { path: "/companies/123", label: "Company Route" },
    { path: "/companies/123/edit", label: "Edit Company Route" },
    { path: "/jobs", label: "Jobs Route" },
    { path: "/payments", label: "Payments Route" },
    { path: "/settings", label: "Settings Route" },
    { path: "/auth/sign-up", label: "Sign Up Route" },
    { path: "/auth/log-in", label: "Log In Route" },
    { path: "/auth/forgot-password", label: "Forgot Password Route" },
    { path: "/auth/confirm-email", label: "Confirm Email Route" },
    { path: "/auth/verify-reset-code", label: "Verify Reset Code Route" },
    { path: "/auth/set-password", label: "Set Password Route" },
];

const withMemoryRouter: Decorator = (Story, context) => {
    const parameters = context.parameters as StoryTestParameters;
    const storyPath = parameters.storyTest?.router?.storyPath || "/";
    const initialEntries = parameters.storyTest?.router?.initialEntries || [storyPath];
    const rootRoute = createRootRoute({
        component: () => <Outlet />,
    });
    const storyRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: storyPath,
        component: () => <Story />,
    });
    const additionalRoutes = routeStubs
        .filter((route) => route.path !== storyPath)
        .map((route) =>
            createRoute({
                getParentRoute: () => rootRoute,
                path: route.path,
                component: () => <div>{route.label}</div>,
            }),
        );
    const routeTree = rootRoute.addChildren([storyRoute, ...additionalRoutes]);
    const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries }),
        context: {},
    });

    return <RouterProvider router={router} />;
};

export default withMemoryRouter;
