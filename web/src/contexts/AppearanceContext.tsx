import {
    createContext,
    useCallback,
    useContext,
    useEffect,
    useMemo,
    useState,
} from "react";
import type { ReactNode } from "react";
import type {
    AppearancePreferences,
    AppearanceStorage,
    ColorPalette,
    ResolvedTheme,
    ThemeMode,
} from "@/lib/appearance";
import {
    applyAppearancePreferences,
    getSystemTheme,
    loadAppearancePreferences,
    persistAppearancePreferences,
    resolveThemeMode,
    subscribeToSystemTheme,
} from "@/lib/appearance";

type AppearanceContextValue = {
    mode: ThemeMode;
    palette: ColorPalette;
    resolvedTheme: ResolvedTheme;
    setMode: (mode: ThemeMode) => void;
};

type AppearanceProviderProps = {
    children: ReactNode;
    initialPreferences?: AppearancePreferences;
    persist?: boolean;
    storage?: AppearanceStorage | null;
};

export const AppearanceContext = createContext<AppearanceContextValue | null>(
    null,
);

export function AppearanceProvider({
    children,
    initialPreferences,
    persist = true,
    storage,
}: AppearanceProviderProps) {
    const [preferences, setPreferences] = useState<AppearancePreferences>(() => {
        if (initialPreferences) {
            return initialPreferences;
        }

        return loadAppearancePreferences(storage);
    });
    const [systemTheme, setSystemTheme] = useState<ResolvedTheme>(getSystemTheme);
    const resolvedTheme = resolveThemeMode(preferences.mode, systemTheme);

    useEffect(() => {
        applyAppearancePreferences(preferences, systemTheme);
    }, [preferences, systemTheme]);

    useEffect(() => {
        if (!persist) {
            return;
        }

        persistAppearancePreferences(preferences, storage);
    }, [persist, preferences, storage]);

    useEffect(() => {
        if (preferences.mode !== "system") {
            return;
        }

        setSystemTheme(getSystemTheme());

        return subscribeToSystemTheme((nextSystemTheme) => {
            setSystemTheme(nextSystemTheme);
        });
    }, [preferences.mode]);

    const setMode = useCallback((mode: ThemeMode) => {
        setPreferences((previous) => {
            if (previous.mode === mode) {
                return previous;
            }

            return {
                ...previous,
                mode,
            };
        });
    }, []);

    const value = useMemo(
        () => ({
            mode: preferences.mode,
            palette: preferences.palette,
            resolvedTheme,
            setMode,
        }),
        [preferences.mode, preferences.palette, resolvedTheme, setMode],
    );

    return (
        <AppearanceContext.Provider value={value}>
            {children}
        </AppearanceContext.Provider>
    );
}

export function useAppearance() {
    const context = useContext(AppearanceContext);

    if (!context) {
        throw new Error("useAppearance must be used within an AppearanceProvider");
    }

    return context;
}
