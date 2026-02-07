/**
 * Type contracts for Storybook behavior-test parameters.
 *
 * Covered behavior:
 * - Defines `storyTest.auth` overrides for auth-dependent route/page scenarios.
 * - Defines `storyTest.router` overrides for memory-history entry/path control.
 * - Defines `storyTest.spies` hooks for asserting non-visual side effects.
 */
import type { ContextType } from "react";
import type { NotificationProps } from "@/components/core/Notification/Notification";
import { AuthContext } from "@/contexts/AuthContext";

export type AuthContextValue = NonNullable<ContextType<typeof AuthContext>>;

export type StoryAuthOverrides = Partial<
    Pick<
        AuthContextValue,
        "user" | "isLoggedIn" | "isLoading" | "refreshUser" | "setUser"
    >
>;

export type StoryRouterOverrides = {
    initialEntries?: string[];
    storyPath?: string;
};

export type StorySpyOverrides = {
    addNotification?: (
        notification: Omit<NotificationProps, "onClose">,
    ) => void;
};

export type StoryTestConfig = {
    auth?: StoryAuthOverrides;
    router?: StoryRouterOverrides;
    spies?: StorySpyOverrides;
};

export type StoryTestParameters = {
    storyTest?: StoryTestConfig;
};
