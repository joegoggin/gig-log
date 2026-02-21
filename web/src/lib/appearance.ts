export type ThemeMode = "system" | "light" | "dark";

export type ResolvedTheme = "light" | "dark";

export const COLOR_PALETTES = ["default", "sunset", "forest"] as const;

export type ColorPalette = (typeof COLOR_PALETTES)[number];

export type AppearancePreferences = {
    mode: ThemeMode;
    palette: ColorPalette;
};

export type PaletteSeedHexColors = {
    background_seed_hex: string;
    text_seed_hex: string;
    primary_seed_hex: string;
    secondary_seed_hex: string;
    green_seed_hex: string;
    red_seed_hex: string;
    yellow_seed_hex: string;
    blue_seed_hex: string;
    magenta_seed_hex: string;
    cyan_seed_hex: string;
};

export type PaletteRgbTokens = {
    background: string;
    text: string;
    primary_100: string;
    primary_80: string;
    primary_60: string;
    secondary_100: string;
    secondary_80: string;
    secondary_60: string;
    black: string;
    white: string;
    green_100: string;
    green_80: string;
    green_60: string;
    red_100: string;
    red_80: string;
    red_60: string;
    yellow_100: string;
    yellow_80: string;
    yellow_60: string;
    blue_100: string;
    blue_80: string;
    blue_60: string;
    magenta_100: string;
    magenta_80: string;
    magenta_60: string;
    cyan_100: string;
    cyan_80: string;
    cyan_60: string;
};

export type PaletteAttribute = ColorPalette | "custom";

export type AppearanceStorage = Pick<Storage, "getItem" | "setItem">;

export const APPEARANCE_STORAGE_KEY = "giglog.appearance";

export const SYSTEM_COLOR_SCHEME_QUERY = "(prefers-color-scheme: dark)";

export const DEFAULT_APPEARANCE_PREFERENCES: AppearancePreferences = {
    mode: "system",
    palette: "default",
};

const PALETTE_TOKEN_VARIABLE_NAMES: Record<keyof PaletteRgbTokens, string> = {
    background: "--color-background-rgb",
    text: "--color-text-rgb",
    primary_100: "--color-primary-100-rgb",
    primary_80: "--color-primary-80-rgb",
    primary_60: "--color-primary-60-rgb",
    secondary_100: "--color-secondary-100-rgb",
    secondary_80: "--color-secondary-80-rgb",
    secondary_60: "--color-secondary-60-rgb",
    black: "--color-black-rgb",
    white: "--color-white-rgb",
    green_100: "--color-green-100-rgb",
    green_80: "--color-green-80-rgb",
    green_60: "--color-green-60-rgb",
    red_100: "--color-red-100-rgb",
    red_80: "--color-red-80-rgb",
    red_60: "--color-red-60-rgb",
    yellow_100: "--color-yellow-100-rgb",
    yellow_80: "--color-yellow-80-rgb",
    yellow_60: "--color-yellow-60-rgb",
    blue_100: "--color-blue-100-rgb",
    blue_80: "--color-blue-80-rgb",
    blue_60: "--color-blue-60-rgb",
    magenta_100: "--color-magenta-100-rgb",
    magenta_80: "--color-magenta-80-rgb",
    magenta_60: "--color-magenta-60-rgb",
    cyan_100: "--color-cyan-100-rgb",
    cyan_80: "--color-cyan-80-rgb",
    cyan_60: "--color-cyan-60-rgb",
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

const isValidHexColor = (value: string): boolean => {
    return /^#[\da-fA-F]{6}$/.test(value.trim());
};

const parseHexChannel = (value: string): number => {
    const channel = Number.parseInt(value, 16);

    if (Number.isNaN(channel)) {
        throw new Error("Invalid hex color channel");
    }

    return channel;
};

const parseHexToRgbChannels = (hex: string): [number, number, number] => {
    const normalized = hex.trim();

    if (!isValidHexColor(normalized)) {
        throw new Error("Color must use 6-digit hex format.");
    }

    return [
        parseHexChannel(normalized.slice(1, 3)),
        parseHexChannel(normalized.slice(3, 5)),
        parseHexChannel(normalized.slice(5, 7)),
    ];
};

const lightenChannel = (channel: number, mixWithWhite: number): number => {
    const mixed = channel + (255 - channel) * mixWithWhite;
    return Math.round(Math.max(0, Math.min(255, mixed)));
};

const toRgbTriplet = (channels: [number, number, number]): string => {
    return `${channels[0]}, ${channels[1]}, ${channels[2]}`;
};

const shadeHexColor = (hex: string, mixWithWhite: number): string => {
    const [red, green, blue] = parseHexToRgbChannels(hex);

    return toRgbTriplet([
        lightenChannel(red, mixWithWhite),
        lightenChannel(green, mixWithWhite),
        lightenChannel(blue, mixWithWhite),
    ]);
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

const parseStoredPreferences = (
    storedValue: unknown,
): AppearancePreferences => {
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
    if (
        typeof window === "undefined" ||
        typeof window.matchMedia !== "function"
    ) {
        return "light";
    }

    return window.matchMedia(SYSTEM_COLOR_SCHEME_QUERY).matches
        ? "dark"
        : "light";
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
    palette: PaletteAttribute,
    target: HTMLElement | null = getDocumentElement(),
): void {
    if (!target) {
        return;
    }

    target.setAttribute("data-palette", palette);
}

export function generatePaletteTokensFromSeeds(
    seedColors: PaletteSeedHexColors,
): PaletteRgbTokens {
    const background = shadeHexColor(seedColors.background_seed_hex, 0);
    const text = shadeHexColor(seedColors.text_seed_hex, 0);

    return {
        background,
        text,
        primary_100: shadeHexColor(seedColors.primary_seed_hex, 0),
        primary_80: shadeHexColor(seedColors.primary_seed_hex, 0.2),
        primary_60: shadeHexColor(seedColors.primary_seed_hex, 0.4),
        secondary_100: shadeHexColor(seedColors.secondary_seed_hex, 0),
        secondary_80: shadeHexColor(seedColors.secondary_seed_hex, 0.2),
        secondary_60: shadeHexColor(seedColors.secondary_seed_hex, 0.4),
        black: text,
        white: background,
        green_100: shadeHexColor(seedColors.green_seed_hex, 0),
        green_80: shadeHexColor(seedColors.green_seed_hex, 0.2),
        green_60: shadeHexColor(seedColors.green_seed_hex, 0.4),
        red_100: shadeHexColor(seedColors.red_seed_hex, 0),
        red_80: shadeHexColor(seedColors.red_seed_hex, 0.2),
        red_60: shadeHexColor(seedColors.red_seed_hex, 0.4),
        yellow_100: shadeHexColor(seedColors.yellow_seed_hex, 0),
        yellow_80: shadeHexColor(seedColors.yellow_seed_hex, 0.2),
        yellow_60: shadeHexColor(seedColors.yellow_seed_hex, 0.4),
        blue_100: shadeHexColor(seedColors.blue_seed_hex, 0),
        blue_80: shadeHexColor(seedColors.blue_seed_hex, 0.2),
        blue_60: shadeHexColor(seedColors.blue_seed_hex, 0.4),
        magenta_100: shadeHexColor(seedColors.magenta_seed_hex, 0),
        magenta_80: shadeHexColor(seedColors.magenta_seed_hex, 0.2),
        magenta_60: shadeHexColor(seedColors.magenta_seed_hex, 0.4),
        cyan_100: shadeHexColor(seedColors.cyan_seed_hex, 0),
        cyan_80: shadeHexColor(seedColors.cyan_seed_hex, 0.2),
        cyan_60: shadeHexColor(seedColors.cyan_seed_hex, 0.4),
    };
}

export function applyCustomPaletteTokens(
    tokens: PaletteRgbTokens,
    target: HTMLElement | null = getDocumentElement(),
): void {
    if (!target) {
        return;
    }

    for (const token of Object.keys(PALETTE_TOKEN_VARIABLE_NAMES) as Array<
        keyof PaletteRgbTokens
    >) {
        target.style.setProperty(
            PALETTE_TOKEN_VARIABLE_NAMES[token],
            tokens[token],
        );
    }
}

export function clearCustomPaletteTokens(
    target: HTMLElement | null = getDocumentElement(),
): void {
    if (!target) {
        return;
    }

    for (const variableName of Object.values(PALETTE_TOKEN_VARIABLE_NAMES)) {
        target.style.removeProperty(variableName);
    }
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
    if (
        typeof window === "undefined" ||
        typeof window.matchMedia !== "function"
    ) {
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
