export type ThemeMode = "system" | "light" | "dark";

export type ResolvedTheme = "light" | "dark";

export const COLOR_PALETTES = ["default", "sunset", "forest"] as const;

export type ColorPalette = (typeof COLOR_PALETTES)[number];

export type AppearancePreferences = {
    mode: ThemeMode;
    palette: ColorPalette;
};

export type AppearanceStorage = Pick<Storage, "getItem" | "setItem">;

export const APPEARANCE_STORAGE_KEY = "giglog.appearance";

export const SYSTEM_COLOR_SCHEME_QUERY = "(prefers-color-scheme: dark)";

export const DEFAULT_APPEARANCE_PREFERENCES: AppearancePreferences = {
    mode: "system",
    palette: "default",
};

const isRecord = (value: unknown): value is Record<string, unknown> => {
    return typeof value === "object" && value !== null;
};

const isThemeMode = (value: unknown): value is ThemeMode => {
    return value === "system" || value === "light" || value === "dark";
};

const isColorPalette = (value: unknown): value is ColorPalette => {
    return (
        typeof value === "string" &&
        COLOR_PALETTES.includes(value as ColorPalette)
    );
};

const getDefaultStorage = (): AppearanceStorage | null => {
    if (typeof window === "undefined") {
        return null;
    }

    try {
        return window.localStorage;
    } catch {
        return null;
    }
};

const getDocumentElement = (): HTMLElement | null => {
    if (typeof document === "undefined") {
        return null;
    }

    return document.documentElement;
};

const parseStoredPreferences = (storedValue: unknown): AppearancePreferences => {
    if (!isRecord(storedValue)) {
        return DEFAULT_APPEARANCE_PREFERENCES;
    }

    return {
        mode: isThemeMode(storedValue.mode)
            ? storedValue.mode
            : DEFAULT_APPEARANCE_PREFERENCES.mode,
        palette: isColorPalette(storedValue.palette)
            ? storedValue.palette
            : DEFAULT_APPEARANCE_PREFERENCES.palette,
    };
};

export function getSystemTheme(): ResolvedTheme {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
        return "light";
    }

    return window.matchMedia(SYSTEM_COLOR_SCHEME_QUERY).matches ? "dark" : "light";
}

export function resolveThemeMode(
    mode: ThemeMode,
    systemTheme: ResolvedTheme,
): ResolvedTheme {
    if (mode === "system") {
        return systemTheme;
    }

    return mode;
}

export function loadAppearancePreferences(
    storage: AppearanceStorage | null = getDefaultStorage(),
): AppearancePreferences {
    if (!storage) {
        return DEFAULT_APPEARANCE_PREFERENCES;
    }

    try {
        const rawPreferences = storage.getItem(APPEARANCE_STORAGE_KEY);

        if (!rawPreferences) {
            return DEFAULT_APPEARANCE_PREFERENCES;
        }

        const parsedPreferences = JSON.parse(rawPreferences) as unknown;

        return parseStoredPreferences(parsedPreferences);
    } catch {
        return DEFAULT_APPEARANCE_PREFERENCES;
    }
}

export function persistAppearancePreferences(
    preferences: AppearancePreferences,
    storage: AppearanceStorage | null = getDefaultStorage(),
): void {
    if (!storage) {
        return;
    }

    try {
        storage.setItem(APPEARANCE_STORAGE_KEY, JSON.stringify(preferences));
    } catch {
        // Ignore persistence failures (private mode, blocked storage, etc.).
    }
}

export function applyTheme(
    theme: ResolvedTheme,
    target: HTMLElement | null = getDocumentElement(),
): void {
    if (!target) {
        return;
    }

    target.setAttribute("data-theme", theme);
}

export function applyPalette(
    palette: ColorPalette,
    target: HTMLElement | null = getDocumentElement(),
): void {
    if (!target) {
        return;
    }

    target.setAttribute("data-palette", palette);
}

export function applyAppearancePreferences(
    preferences: AppearancePreferences,
    systemTheme: ResolvedTheme = getSystemTheme(),
    target: HTMLElement | null = getDocumentElement(),
): ResolvedTheme {
    const resolvedTheme = resolveThemeMode(preferences.mode, systemTheme);

    applyTheme(resolvedTheme, target);
    applyPalette(preferences.palette, target);

    return resolvedTheme;
}

export function initializeAppearance(
    storage: AppearanceStorage | null = getDefaultStorage(),
    target: HTMLElement | null = getDocumentElement(),
): AppearancePreferences {
    const preferences = loadAppearancePreferences(storage);
    applyAppearancePreferences(preferences, getSystemTheme(), target);

    return preferences;
}

export function subscribeToSystemTheme(
    onThemeChange: (theme: ResolvedTheme) => void,
): () => void {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
        return () => {};
    }

    const mediaQuery = window.matchMedia(SYSTEM_COLOR_SCHEME_QUERY);
    const listener = (event: MediaQueryListEvent) => {
        onThemeChange(event.matches ? "dark" : "light");
    };

    if (
        typeof mediaQuery.addEventListener === "function" &&
        typeof mediaQuery.removeEventListener === "function"
    ) {
        mediaQuery.addEventListener("change", listener);
        return () => {
            mediaQuery.removeEventListener("change", listener);
        };
    }

    if (
        typeof mediaQuery.addListener === "function" &&
        typeof mediaQuery.removeListener === "function"
    ) {
        mediaQuery.addListener(listener);
        return () => {
            mediaQuery.removeListener(listener);
        };
    }

    return () => {};
}
