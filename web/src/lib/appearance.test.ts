/**
 * Unit tests for appearance preference persistence and boot-time appearance setup.
 *
 * Covered scenarios:
 * - Default appearance preferences are used when storage is empty or invalid.
 * - Saved theme mode and palette preferences persist and restore correctly.
 * - App initialization applies the restored preference to `data-theme`/`data-palette`.
 * - Custom palette seed colors generate deterministic token shades and apply/clear CSS vars.
 *
 * These tests prevent regressions where a saved theme mode is ignored after
 * reload or malformed storage data breaks appearance initialization.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AppearanceStorage } from "@/lib/appearance";
import {
    APPEARANCE_STORAGE_KEY,
    DEFAULT_APPEARANCE_PREFERENCES,
    applyCustomPaletteTokens,
    clearCustomPaletteTokens,
    generatePaletteTokensFromSeeds,
    initializeAppearance,
    loadAppearancePreferences,
    persistAppearancePreferences,
} from "@/lib/appearance";

const createMemoryStorage = (): AppearanceStorage => {
    const state = new Map<string, string>();

    return {
        getItem: (key: string) => {
            return state.get(key) ?? null;
        },
        setItem: (key: string, value: string) => {
            state.set(key, value);
        },
    };
};

const mockMatchMedia = (initialMatches: boolean) => {
    const originalMatchMedia = window.matchMedia;
    let matches = initialMatches;

    Object.defineProperty(window, "matchMedia", {
        writable: true,
        value: vi.fn().mockImplementation(() => ({
            matches,
            media: "(prefers-color-scheme: dark)",
            onchange: null,
            addListener: vi.fn(),
            removeListener: vi.fn(),
            addEventListener: vi.fn(),
            removeEventListener: vi.fn(),
            dispatchEvent: vi.fn(),
        })),
    });

    return {
        setMatches: (nextMatches: boolean) => {
            matches = nextMatches;
        },
        restore: () => {
            Object.defineProperty(window, "matchMedia", {
                writable: true,
                value: originalMatchMedia,
            });
        },
    };
};

describe("appearance helpers", () => {
    let storage: AppearanceStorage;

    beforeEach(() => {
        storage = createMemoryStorage();
        document.documentElement.removeAttribute("data-theme");
        document.documentElement.removeAttribute("data-palette");
    });

    afterEach(() => {
        vi.restoreAllMocks();
    });

    it("uses default appearance when no preference is stored", () => {
        expect(loadAppearancePreferences(storage)).toEqual(
            DEFAULT_APPEARANCE_PREFERENCES,
        );
    });

    it("falls back to default appearance when stored mode is invalid", () => {
        storage.setItem(
            APPEARANCE_STORAGE_KEY,
            JSON.stringify({ mode: "invalid-mode", palette: "default" }),
        );

        expect(loadAppearancePreferences(storage)).toEqual(
            DEFAULT_APPEARANCE_PREFERENCES,
        );
    });

    it("falls back to default appearance when stored palette is invalid", () => {
        storage.setItem(
            APPEARANCE_STORAGE_KEY,
            JSON.stringify({ mode: "dark", palette: "invalid-palette" }),
        );

        expect(loadAppearancePreferences(storage)).toEqual({
            ...DEFAULT_APPEARANCE_PREFERENCES,
            mode: "dark",
        });
    });

    it("restores persisted appearance preference", () => {
        const savedPreferences = {
            mode: "dark",
            palette: "sunset",
        } as const;

        persistAppearancePreferences(savedPreferences, storage);

        expect(loadAppearancePreferences(storage)).toEqual(savedPreferences);
    });

    it("applies persisted mode to data-theme on initialize", () => {
        const mediaQueryMock = mockMatchMedia(false);

        try {
            persistAppearancePreferences(
                { mode: "dark", palette: "forest" },
                storage,
            );

            initializeAppearance(storage);

            expect(document.documentElement.getAttribute("data-theme")).toBe(
                "dark",
            );
            expect(document.documentElement.getAttribute("data-palette")).toBe(
                "forest",
            );
        } finally {
            mediaQueryMock.restore();
        }
    });

    it("uses current system preference when mode is system", () => {
        const mediaQueryMock = mockMatchMedia(true);

        try {
            persistAppearancePreferences(
                { mode: "system", palette: "default" },
                storage,
            );

            initializeAppearance(storage);
            expect(document.documentElement.getAttribute("data-theme")).toBe(
                "dark",
            );

            mediaQueryMock.setMatches(false);
            initializeAppearance(storage);
            expect(document.documentElement.getAttribute("data-theme")).toBe(
                "light",
            );
        } finally {
            mediaQueryMock.restore();
        }
    });

    it("generates deterministic accent shades from seed colors", () => {
        const tokens = generatePaletteTokensFromSeeds({
            background_seed_hex: "#a9b1d6",
            text_seed_hex: "#1a1b26",
            primary_seed_hex: "#9ece6a",
            secondary_seed_hex: "#7aa2f7",
            green_seed_hex: "#336699",
            red_seed_hex: "#e65100",
            yellow_seed_hex: "#f9a825",
            blue_seed_hex: "#1e88e5",
            magenta_seed_hex: "#8e24aa",
            cyan_seed_hex: "#00838f",
        });

        expect(tokens.green_100).toBe("51, 102, 153");
        expect(tokens.green_80).toBe("92, 133, 173");
        expect(tokens.green_60).toBe("133, 163, 194");
        expect(tokens.background).toBe("169, 177, 214");
        expect(tokens.text).toBe("26, 27, 38");
        expect(tokens.primary_100).toBe("158, 206, 106");
        expect(tokens.secondary_100).toBe("122, 162, 247");
        expect(tokens.black).toBe("26, 27, 38");
        expect(tokens.white).toBe("169, 177, 214");
    });

    it("applies and clears custom palette css variables", () => {
        const tokens = generatePaletteTokensFromSeeds({
            background_seed_hex: "#a9b1d6",
            text_seed_hex: "#1a1b26",
            primary_seed_hex: "#9ece6a",
            secondary_seed_hex: "#7aa2f7",
            green_seed_hex: "#66bb6a",
            red_seed_hex: "#e27d7c",
            yellow_seed_hex: "#d0a761",
            blue_seed_hex: "#5c93cd",
            magenta_seed_hex: "#a082ce",
            cyan_seed_hex: "#59b7aa",
        });

        applyCustomPaletteTokens(tokens, document.documentElement);

        expect(
            document.documentElement.style.getPropertyValue(
                "--color-blue-100-rgb",
            ),
        ).toBe(tokens.blue_100);

        clearCustomPaletteTokens(document.documentElement);

        expect(
            document.documentElement.style.getPropertyValue(
                "--color-blue-100-rgb",
            ),
        ).toBe("");
    });
});
