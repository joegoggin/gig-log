/**
 * Type contracts for Storybook behavior-test parameters.
 *
 * Covered behavior:
 * - Defines `storyTest.auth` overrides for auth-dependent route/page scenarios.
 * - Defines `storyTest.router` overrides for memory-history entry/path control.
 * - Defines `storyTest.spies` hooks for asserting non-visual side effects.
 * - Defines `storyTest.appearance` overrides for deterministic theme and palette behavior.
 */
import type { ContextType } from "react";
import type { NotificationProps } from "@/components/core/Notification/Notification";
import type { AuthContext } from "@/contexts/AuthContext";
import type { ColorPalette, ThemeMode } from "@/lib/appearance";

export type AuthContextValue = NonNullable<ContextType<typeof AuthContext>>;

export type StoryAuthOverrides = Partial<
    Pick<
        AuthContextValue,
        "user" | "isLoggedIn" | "isLoading" | "refreshUser" | "setUser"
    >
>;

export type StoryRouterOverrides = {
    initialEntries?: Array<string>;
    storyPath?: string;
};

export type StorySpyOverrides = {
    addNotification?: (
        notification: Omit<NotificationProps, "onClose">,
    ) => void;
};

export type StoryAppearanceOverrides = {
    mode?: ThemeMode;
    palette?: ColorPalette;
};

export type StoryTestConfig = {
    auth?: StoryAuthOverrides;
    router?: StoryRouterOverrides;
    spies?: StorySpyOverrides;
    appearance?: StoryAppearanceOverrides;
};

export type StoryTestParameters = {
    storyTest?: StoryTestConfig;
};
