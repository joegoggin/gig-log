/**
 * Storybook decorator for app-level provider setup in behavior tests.
 *
 * Covered behavior:
 * - Injects isolated QueryClient for deterministic mutation/query behavior.
 * - Injects auth context overrides from `parameters.storyTest.auth`.
 * - Captures notification calls via `parameters.storyTest.spies.addNotification`.
 * - Keeps real notification rendering so stories can assert visible messages.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
    
    
    useCallback,
    useMemo,
    useRef,
    useState
} from "react";
import type {ContextType, ReactNode} from "react";
import type { Decorator } from "@storybook/react-vite";
import type { NotificationProps } from "@/components/core/Notification/Notification";
import type {
    StoryTestConfig,
    StoryTestParameters,
} from "@/stories/testing/storyTestContext";
import { AuthContext } from "@/contexts/AuthContext";
import { NotificationContext } from "@/contexts/NotificationContext";

type StoryProvidersProps = {
    children: ReactNode;
    config?: StoryTestConfig;
};

type NotificationWithId = NotificationProps & {
    id: string;
};

const createBaseAuthValue = (): NonNullable<ContextType<typeof AuthContext>> => ({
    user: null,
    isLoggedIn: false,
    isLoading: false,
    refreshUser: async () => {},
    setUser: () => {},
});

function StoryProviders({ children, config }: StoryProvidersProps) {
    const [queryClient] = useState(
        () =>
            new QueryClient({
                defaultOptions: {
                    queries: {
                        retry: false,
                    },
                    mutations: {
                        retry: false,
                    },
                },
            }),
    );
    const [notifications, setNotifications] = useState<Array<NotificationWithId>>([]);
    const notificationIdRef = useRef(0);
    const authOverrides = config?.auth;
    const authValue = useMemo(() => {
        const baseAuthValue = createBaseAuthValue();
        const user = authOverrides?.user ?? baseAuthValue.user;

        return {
            ...baseAuthValue,
            ...authOverrides,
            user,
            isLoggedIn: authOverrides?.isLoggedIn ?? Boolean(user),
        };
    }, [authOverrides]);

    const addNotification = useCallback(
        (notification: Omit<NotificationProps, "onClose">) => {
            config?.spies?.addNotification?.(notification);
            const id = `story-notification-${notificationIdRef.current}`;
            notificationIdRef.current += 1;
            setNotifications((prev) => [...prev, { ...notification, id }]);
        },
        [config],
    );

    const removeNotification = useCallback((id: string) => {
        setNotifications((prev) => prev.filter((notification) => notification.id !== id));
    }, []);

    return (
        <QueryClientProvider client={queryClient}>
            <AuthContext.Provider value={authValue}>
                <NotificationContext.Provider
                    value={{ notifications, addNotification, removeNotification }}
                >
                    {children}
                </NotificationContext.Provider>
            </AuthContext.Provider>
        </QueryClientProvider>
    );
}

const withAppProviders: Decorator = (Story, context) => {
    const parameters = context.parameters as StoryTestParameters;

    return (
        <StoryProviders config={parameters.storyTest}>
            <Story />
        </StoryProviders>
    );
};

export default withAppProviders;
