import {
    Outlet,
    RouterProvider,
    createMemoryHistory,
    createRootRoute,
    createRoute,
    createRouter,
} from "@tanstack/react-router";
import type { Decorator } from "@storybook/react-vite";

const withMemoryRouter: Decorator = (Story) => {
    const rootRoute = createRootRoute({
        component: () => <Outlet />,
    });
    const storyRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: "/",
        component: () => <Story />,
    });
    const dashboardRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: "/dashboard",
        component: () => <div>Dashboard Route</div>,
    });
    const signUpRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: "/auth/sign-up",
        component: () => <div>Sign Up Route</div>,
    });
    const logInRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: "/auth/log-in",
        component: () => <div>Log In Route</div>,
    });
    const routeTree = rootRoute.addChildren([
        storyRoute,
        dashboardRoute,
        signUpRoute,
        logInRoute,
    ]);
    const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ["/"] }),
        context: {},
    });

    return <RouterProvider router={router} />;
};

export default withMemoryRouter;
