import { Outlet, createRootRoute } from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";
import { TanStackDevtools } from "@tanstack/react-devtools";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "@sass/index.scss";

const queryClient = new QueryClient();

/**
 * The root component that wraps all pages in the application.
 * Sets up the QueryClientProvider for React Query and includes TanStack
 * devtools for development debugging.
 */
function RootComponent() {
    return (
        <QueryClientProvider client={queryClient}>
            <Outlet />
            <TanStackDevtools
                config={{
                    position: "bottom-right",
                }}
                plugins={[
                    {
                        name: "Tanstack Router",
                        render: <TanStackRouterDevtoolsPanel />,
                    },
                ]}
            />
        </QueryClientProvider>
    );
}

export const Route = createRootRoute({
    component: RootComponent,
});
