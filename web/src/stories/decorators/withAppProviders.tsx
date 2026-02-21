/**
 * Storybook decorator for app-level provider setup in behavior tests.
 *
 * Covered behavior:
 * - Injects isolated QueryClient for deterministic mutation/query behavior.
 * - Injects auth context overrides from `parameters.storyTest.auth`.
 * - Captures notification calls via `parameters.storyTest.spies.addNotification`.
 * - Applies deterministic appearance defaults (theme + palette).
 * - Allows opting into persisted appearance API behavior per story.
 * - Keeps real notification rendering so stories can assert visible messages.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useCallback, useMemo, useRef, useState } from "react";
import type { ContextType, ReactNode } from "react";
import type { Decorator } from "@storybook/react-vite";
import type { NotificationProps } from "@/components/core/Notification/Notification";
import type {
    StoryTestConfig,
    StoryTestParameters,
} from "@/stories/testing/storyTestContext";
import type { AppearancePreferences } from "@/lib/appearance";
import { DEFAULT_APPEARANCE_PREFERENCES } from "@/lib/appearance";
import { AuthContext } from "@/contexts/AuthContext";
import { AppearanceProvider } from "@/contexts/AppearanceContext";
import { NotificationContext } from "@/contexts/NotificationContext";
import { WorkSessionProvider } from "@/contexts/WorkSessionContext";

type StoryProvidersProps = {
    children: ReactNode;
    config?: StoryTestConfig;
};

type NotificationWithId = NotificationProps & {
    id: string;
};

const createBaseAuthValue = (): NonNullable<
    ContextType<typeof AuthContext>
> => ({
    user: null,
    isLoggedIn: false,
    isLoading: false,
    refreshUser: async () => {},
    setUser: () => {},
});

const createAppearancePreferences = (
    config?: StoryTestConfig,
): AppearancePreferences => {
    return {
        ...DEFAULT_APPEARANCE_PREFERENCES,
        mode: config?.appearance?.mode ?? "dark",
        palette:
            config?.appearance?.palette ??
            DEFAULT_APPEARANCE_PREFERENCES.palette,
    };
};

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
    const [notifications, setNotifications] = useState<
        Array<NotificationWithId>
    >([]);
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
    const appearancePreferences = useMemo(
        () => createAppearancePreferences(config),
        [config],
    );
    const appearancePersist = config?.appearance?.persist ?? false;

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
        setNotifications((prev) =>
            prev.filter((notification) => notification.id !== id),
        );
    }, []);

    return (
        <QueryClientProvider client={queryClient}>
            <AuthContext.Provider value={authValue}>
                <AppearanceProvider
                    initialPreferences={appearancePreferences}
                    persist={appearancePersist}
                >
                    <NotificationContext.Provider
                        value={{
                            notifications,
                            addNotification,
                            removeNotification,
                        }}
                    >
                        <WorkSessionProvider>{children}</WorkSessionProvider>
                    </NotificationContext.Provider>
                </AppearanceProvider>
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
