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
    ActivePaletteSelection,
    AppearanceSettingsResponse,
    CreateCustomPaletteRequest,
    CreateCustomPaletteResponse,
    CustomPalette,
    SetActivePaletteResponse,
} from "@/types/models/Appearance";
import type {
    AppearancePreferences,
    AppearanceStorage,
    ColorPalette,
    ResolvedTheme,
    ThemeMode,
} from "@/lib/appearance";
import {
    DEFAULT_APPEARANCE_PREFERENCES,
    applyCustomPaletteTokens,
    applyPalette,
    applyTheme,
    clearCustomPaletteTokens,
    generatePaletteTokensFromSeeds,
    getSystemTheme,
    loadAppearancePreferences,
    persistAppearancePreferences,
    resolveThemeMode,
    subscribeToSystemTheme,
} from "@/lib/appearance";
import { useAuth } from "@/contexts/AuthContext";
import api from "@/lib/axios";

type AppearanceContextValue = {
    mode: ThemeMode;
    resolvedTheme: ResolvedTheme;
    activePalette: ActivePaletteSelection;
    customPalettes: Array<CustomPalette>;
    setMode: (mode: ThemeMode) => void;
    selectPresetPalette: (palette: ColorPalette) => Promise<void>;
    selectCustomPalette: (customPaletteId: string) => Promise<void>;
    createCustomPalette: (
        payload: CreateCustomPaletteRequest,
    ) => Promise<CustomPalette>;
    refreshPalettes: () => Promise<void>;
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

const createPresetSelection = (
    palette: ColorPalette,
): ActivePaletteSelection => {
    return {
        palette_type: "preset",
        preset_palette: palette,
        custom_palette_id: null,
    };
};

const isPresetPalette = (palette: string): palette is ColorPalette => {
    return (
        palette === "default" || palette === "sunset" || palette === "forest"
    );
};

const normalizeActivePaletteSelection = (
    activePalette: ActivePaletteSelection,
    customPalettes: Array<CustomPalette>,
): ActivePaletteSelection => {
    if (
        activePalette.palette_type === "custom" &&
        activePalette.custom_palette_id
    ) {
        const exists = customPalettes.some(
            (palette) => palette.id === activePalette.custom_palette_id,
        );

        if (exists) {
            return {
                palette_type: "custom",
                preset_palette: null,
                custom_palette_id: activePalette.custom_palette_id,
            };
        }
    }

    if (
        activePalette.palette_type === "preset" &&
        typeof activePalette.preset_palette === "string" &&
        isPresetPalette(activePalette.preset_palette)
    ) {
        return createPresetSelection(activePalette.preset_palette);
    }

    return createPresetSelection(DEFAULT_APPEARANCE_PREFERENCES.palette);
};

const createLocalPaletteId = () => {
    try {
        return globalThis.crypto.randomUUID();
    } catch {
        return `local-palette-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
    }
};

const normalizeCreatePalettePayload = (
    payload: CreateCustomPaletteRequest,
): CreateCustomPaletteRequest => {
    return {
        name: payload.name.trim(),
        background_seed_hex: payload.background_seed_hex.trim().toLowerCase(),
        text_seed_hex: payload.text_seed_hex.trim().toLowerCase(),
        primary_seed_hex: payload.primary_seed_hex.trim().toLowerCase(),
        secondary_seed_hex: payload.secondary_seed_hex.trim().toLowerCase(),
        green_seed_hex: payload.green_seed_hex.trim().toLowerCase(),
        red_seed_hex: payload.red_seed_hex.trim().toLowerCase(),
        yellow_seed_hex: payload.yellow_seed_hex.trim().toLowerCase(),
        blue_seed_hex: payload.blue_seed_hex.trim().toLowerCase(),
        magenta_seed_hex: payload.magenta_seed_hex.trim().toLowerCase(),
        cyan_seed_hex: payload.cyan_seed_hex.trim().toLowerCase(),
    };
};

export function AppearanceProvider({
    children,
    initialPreferences,
    persist = true,
    storage,
}: AppearanceProviderProps) {
    const { isLoggedIn } = useAuth();
    const [preferences, setPreferences] = useState<AppearancePreferences>(
        () => {
            if (initialPreferences) {
                return initialPreferences;
            }

            return loadAppearancePreferences(storage);
        },
    );
    const [activePalette, setActivePalette] = useState<ActivePaletteSelection>(
        () =>
            createPresetSelection(
                initialPreferences?.palette ??
                    loadAppearancePreferences(storage).palette,
            ),
    );
    const [customPalettes, setCustomPalettes] = useState<Array<CustomPalette>>(
        [],
    );
    const [systemTheme, setSystemTheme] =
        useState<ResolvedTheme>(getSystemTheme);
    const resolvedTheme = resolveThemeMode(preferences.mode, systemTheme);

    useEffect(() => {
        applyTheme(resolvedTheme);

        if (
            activePalette.palette_type === "custom" &&
            activePalette.custom_palette_id
        ) {
            const selectedCustomPalette = customPalettes.find(
                (palette) => palette.id === activePalette.custom_palette_id,
            );

            if (selectedCustomPalette) {
                applyPalette("custom");
                applyCustomPaletteTokens(
                    selectedCustomPalette.generated_tokens,
                );
                return;
            }
        }

        clearCustomPaletteTokens();
        applyPalette(preferences.palette);
    }, [
        activePalette.custom_palette_id,
        activePalette.palette_type,
        customPalettes,
        preferences.palette,
        resolvedTheme,
    ]);

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

    const refreshPalettes = useCallback(async () => {
        if (!persist || !isLoggedIn) {
            return;
        }

        const response =
            await api.get<AppearanceSettingsResponse>("/appearance");
        const nextCustomPalettes = response.data.custom_palettes;
        const nextActivePalette = normalizeActivePaletteSelection(
            response.data.active_palette,
            nextCustomPalettes,
        );

        setCustomPalettes(nextCustomPalettes);
        setActivePalette(nextActivePalette);

        const presetPalette =
            nextActivePalette.palette_type === "preset"
                ? nextActivePalette.preset_palette
                : null;

        if (presetPalette) {
            setPreferences((previous) => {
                if (previous.palette === presetPalette) {
                    return previous;
                }

                return {
                    ...previous,
                    palette: presetPalette,
                };
            });
        }
    }, [isLoggedIn, persist]);

    useEffect(() => {
        if (!persist) {
            return;
        }

        if (!isLoggedIn) {
            setCustomPalettes([]);
            const fallbackPreferences = loadAppearancePreferences(storage);
            setActivePalette(
                createPresetSelection(fallbackPreferences.palette),
            );
            return;
        }

        void refreshPalettes().catch(() => {
            // Keep local appearance fallback if remote appearance load fails.
        });
    }, [isLoggedIn, persist, refreshPalettes, storage]);

    const selectPresetPalette = useCallback(
        async (palette: ColorPalette) => {
            if (persist && isLoggedIn) {
                await api.put<SetActivePaletteResponse>(
                    "/appearance/active-palette",
                    {
                        palette_type: "preset",
                        preset_palette: palette,
                    },
                );
            }

            setActivePalette(createPresetSelection(palette));
            setPreferences((previous) => {
                if (previous.palette === palette) {
                    return previous;
                }

                return {
                    ...previous,
                    palette,
                };
            });
        },
        [isLoggedIn, persist],
    );

    const selectCustomPalette = useCallback(
        async (customPaletteId: string) => {
            const exists = customPalettes.some(
                (palette) => palette.id === customPaletteId,
            );

            if (!exists) {
                throw new Error("Custom palette not found.");
            }

            if (persist && isLoggedIn) {
                await api.put<SetActivePaletteResponse>(
                    "/appearance/active-palette",
                    {
                        palette_type: "custom",
                        custom_palette_id: customPaletteId,
                    },
                );
            }

            setActivePalette({
                palette_type: "custom",
                preset_palette: null,
                custom_palette_id: customPaletteId,
            });
        },
        [customPalettes, isLoggedIn, persist],
    );

    const createCustomPalette = useCallback(
        async (payload: CreateCustomPaletteRequest) => {
            const normalizedPayload = normalizeCreatePalettePayload(payload);

            if (!normalizedPayload.name) {
                throw new Error("Palette name is required.");
            }

            if (persist && isLoggedIn) {
                const response = await api.post<CreateCustomPaletteResponse>(
                    "/appearance/palettes",
                    normalizedPayload,
                );
                const createdPalette = response.data.palette;
                const nextActivePalette = normalizeActivePaletteSelection(
                    response.data.active_palette,
                    [createdPalette],
                );

                setCustomPalettes((previous) => [
                    createdPalette,
                    ...previous.filter(
                        (palette) => palette.id !== createdPalette.id,
                    ),
                ]);
                setActivePalette(nextActivePalette);

                return createdPalette;
            }

            const nowIso = new Date().toISOString();
            const localPalette: CustomPalette = {
                id: createLocalPaletteId(),
                name: normalizedPayload.name,
                background_seed_hex: normalizedPayload.background_seed_hex,
                text_seed_hex: normalizedPayload.text_seed_hex,
                primary_seed_hex: normalizedPayload.primary_seed_hex,
                secondary_seed_hex: normalizedPayload.secondary_seed_hex,
                green_seed_hex: normalizedPayload.green_seed_hex,
                red_seed_hex: normalizedPayload.red_seed_hex,
                yellow_seed_hex: normalizedPayload.yellow_seed_hex,
                blue_seed_hex: normalizedPayload.blue_seed_hex,
                magenta_seed_hex: normalizedPayload.magenta_seed_hex,
                cyan_seed_hex: normalizedPayload.cyan_seed_hex,
                generated_tokens:
                    generatePaletteTokensFromSeeds(normalizedPayload),
                generation_version: 2,
                created_at: nowIso,
                updated_at: nowIso,
            };

            setCustomPalettes((previous) => [localPalette, ...previous]);
            setActivePalette({
                palette_type: "custom",
                preset_palette: null,
                custom_palette_id: localPalette.id,
            });

            return localPalette;
        },
        [isLoggedIn, persist],
    );

    const value = useMemo(
        () => ({
            mode: preferences.mode,
            resolvedTheme,
            activePalette,
            customPalettes,
            setMode,
            selectPresetPalette,
            selectCustomPalette,
            createCustomPalette,
            refreshPalettes,
        }),
        [
            activePalette,
            createCustomPalette,
            customPalettes,
            preferences.mode,
            refreshPalettes,
            resolvedTheme,
            selectCustomPalette,
            selectPresetPalette,
            setMode,
        ],
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
        throw new Error(
            "useAppearance must be used within an AppearanceProvider",
        );
    }

    return context;
}
