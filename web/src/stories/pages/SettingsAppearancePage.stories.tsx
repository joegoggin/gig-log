/**
 * Storybook interaction tests for appearance settings behavior.
 *
 * Covered scenarios:
 * - Theme mode controls switch appearance and update `data-theme`.
 * - Palette controls restore persisted state and update `data-palette`.
 * - Custom role-color controls apply background/text/primary/secondary tokens.
 * - Custom palette creation adds a selectable palette and activates it.
 * - Custom palette creation requires a non-empty palette name.
 * - Contrast checks verify readable text/surface pairs across palette and theme combinations.
 *
 * These tests prevent regressions in persisted appearance preferences.
 */
import { expect, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { ColorPalette } from "@/lib/appearance";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import SettingsAppearancePage from "@/pages/SettingsAppearancePage/SettingsAppearancePage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const paletteEyebrowColors: Record<ColorPalette, string> = {
    default: "rgba(122, 162, 247, 0.2)",
    sunset: "rgba(103, 154, 245, 0.2)",
    forest: "rgba(92, 147, 205, 0.2)",
};

const parseRgbChannels = (color: string): [number, number, number] => {
    const channels = color.match(/\d+(?:\.\d+)?/g);

    if (!channels || channels.length < 3) {
        throw new Error(`Unable to parse RGB channels from \`${color}\`.`);
    }

    return [Number(channels[0]), Number(channels[1]), Number(channels[2])];
};

const getRelativeLuminance = (color: string): number => {
    const [red, green, blue] = parseRgbChannels(color);
    const normalized = [red, green, blue].map((channel) => {
        const value = channel / 255;
        return value <= 0.03928
            ? value / 12.92
            : ((value + 0.055) / 1.055) ** 2.4;
    });

    return (
        normalized[0] * 0.2126 + normalized[1] * 0.7152 + normalized[2] * 0.0722
    );
};

const getContrastRatio = (foreground: string, background: string): number => {
    const foregroundLuminance = getRelativeLuminance(foreground);
    const backgroundLuminance = getRelativeLuminance(background);
    const lighter = Math.max(foregroundLuminance, backgroundLuminance);
    const darker = Math.min(foregroundLuminance, backgroundLuminance);

    return (lighter + 0.05) / (darker + 0.05);
};

const clickOptionLabel = async (radioInput: HTMLElement) => {
    const label = radioInput.closest("label");

    if (!label) {
        throw new Error(
            "Expected radio input to be wrapped by a label element.",
        );
    }

    await userEvent.click(label);
};

const hexToRgbTriplet = (hex: string): string => {
    const normalized = hex.trim().replace(/^#/, "");
    const red = Number.parseInt(normalized.slice(0, 2), 16);
    const green = Number.parseInt(normalized.slice(2, 4), 16);
    const blue = Number.parseInt(normalized.slice(4, 6), 16);

    return `${red}, ${green}, ${blue}`;
};

const meta: Meta<typeof SettingsAppearancePage> = {
    title: "Pages/SettingsAppearancePage",
    component: SettingsAppearancePage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/settings/appearance",
                initialEntries: ["/settings/appearance"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof SettingsAppearancePage>;

export const SwitchesThemeModeAndUpdatesDataTheme: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/settings/appearance",
                initialEntries: ["/settings/appearance"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
            appearance: {
                mode: "dark",
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        const rootElement = canvasElement.ownerDocument.documentElement;
        const lightModeRadio = canvas.getByRole("radio", { name: /^Light / });
        const darkModeRadio = canvas.getByRole("radio", { name: /^Dark / });

        await expect(darkModeRadio).toBeChecked();
        await expect(rootElement.getAttribute("data-theme")).toBe("dark");

        await clickOptionLabel(lightModeRadio);

        await waitFor(() => {
            expect(rootElement.getAttribute("data-theme")).toBe("light");
        });
        await expect(lightModeRadio).toBeChecked();
    },
};

export const RestoresPalettePreferenceOnLoad: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/settings/appearance",
                initialEntries: ["/settings/appearance"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
            appearance: {
                mode: "light",
                palette: "forest",
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        const rootElement = canvasElement.ownerDocument.documentElement;
        const forestPaletteRadio = canvas.getByRole("radio", {
            name: /^Forest /,
        });
        const eyebrow = canvas.getByText("Account and appearance");

        await expect(rootElement.getAttribute("data-palette")).toBe("forest");
        await expect(forestPaletteRadio).toBeChecked();
        await expect(getComputedStyle(eyebrow).backgroundColor).toBe(
            paletteEyebrowColors.forest,
        );
    },
};

export const SwitchesPaletteAndUpdatesDataPalette: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/settings/appearance",
                initialEntries: ["/settings/appearance"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
            appearance: {
                mode: "light",
                palette: "default",
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        const rootElement = canvasElement.ownerDocument.documentElement;
        const eyebrow = canvas.getByText("Account and appearance");
        const defaultPaletteRadio = canvas.getByRole("radio", {
            name: /^Default /,
        });
        const sunsetPaletteRadio = canvas.getByRole("radio", {
            name: /^Sunset /,
        });
        const forestPaletteRadio = canvas.getByRole("radio", {
            name: /^Forest /,
        });

        await expect(rootElement.getAttribute("data-palette")).toBe("default");
        await expect(defaultPaletteRadio).toBeChecked();
        await expect(getComputedStyle(eyebrow).backgroundColor).toBe(
            paletteEyebrowColors.default,
        );

        await clickOptionLabel(sunsetPaletteRadio);

        await waitFor(() => {
            expect(rootElement.getAttribute("data-palette")).toBe("sunset");
        });
        await expect(sunsetPaletteRadio).toBeChecked();
        await expect(getComputedStyle(eyebrow).backgroundColor).toBe(
            paletteEyebrowColors.sunset,
        );

        await clickOptionLabel(forestPaletteRadio);

        await waitFor(() => {
            expect(rootElement.getAttribute("data-palette")).toBe("forest");
        });
        await expect(forestPaletteRadio).toBeChecked();
        await expect(getComputedStyle(eyebrow).backgroundColor).toBe(
            paletteEyebrowColors.forest,
        );
    },
};

export const MaintainsReadableContrastAcrossPalettesAndThemes: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/settings/appearance",
                initialEntries: ["/settings/appearance"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
            appearance: {
                mode: "light",
                palette: "default",
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        const rootElement = canvasElement.ownerDocument.documentElement;
        const heading = canvas.getByRole("heading", {
            level: 1,
            name: "Appearance settings",
        });
        const themeOptionLabel = canvas.getByText("System");
        const themeOptionContainer = themeOptionLabel.closest("label");

        if (!themeOptionContainer) {
            throw new Error("Expected theme option to be wrapped by a label.");
        }

        const themeSelections = [
            { radioName: /^Light /, value: "light" },
            { radioName: /^Dark /, value: "dark" },
        ] as const;
        const paletteSelections = [
            { radioName: /^Default /, value: "default" },
            { radioName: /^Sunset /, value: "sunset" },
            { radioName: /^Forest /, value: "forest" },
        ] as const;

        for (const themeSelection of themeSelections) {
            const themeRadio = canvas.getByRole("radio", {
                name: themeSelection.radioName,
            });
            await clickOptionLabel(themeRadio);

            await waitFor(() => {
                expect(rootElement.getAttribute("data-theme")).toBe(
                    themeSelection.value,
                );
            });

            for (const paletteSelection of paletteSelections) {
                const paletteRadio = canvas.getByRole("radio", {
                    name: paletteSelection.radioName,
                });
                await clickOptionLabel(paletteRadio);

                await waitFor(() => {
                    expect(rootElement.getAttribute("data-palette")).toBe(
                        paletteSelection.value,
                    );
                });

                const pageContrast = getContrastRatio(
                    getComputedStyle(heading).color,
                    getComputedStyle(canvasElement.ownerDocument.body)
                        .backgroundColor,
                );
                const optionContrast = getContrastRatio(
                    getComputedStyle(themeOptionLabel).color,
                    getComputedStyle(themeOptionContainer).backgroundColor,
                );

                await expect(pageContrast).toBeGreaterThanOrEqual(4.5);
                await expect(optionContrast).toBeGreaterThanOrEqual(4.5);
            }
        }
    },
};

export const CreatesCustomPaletteAndActivatesIt: Story = {
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        const rootElement = canvasElement.ownerDocument.documentElement;
        const paletteNameInput = canvas.getByPlaceholderText("Ocean Mist");
        const backgroundHex = "#a9b1d6";
        const textHex = "#1a1b26";
        const primaryHex = "#9ece6a";
        const secondaryHex = "#7aa2f7";

        await expect(canvas.getByText("Background")).toBeVisible();
        await expect(canvas.getByText("Text")).toBeVisible();
        await expect(canvas.getByText("Primary")).toBeVisible();
        await expect(canvas.getByText("Secondary")).toBeVisible();

        await userEvent.type(paletteNameInput, "Ocean Mist");
        await userEvent.click(
            canvas.getByRole("button", { name: "Create Custom Palette" }),
        );

        const oceanMistRadio = await canvas.findByRole("radio", {
            name: /Ocean Mist/i,
        });

        await waitFor(() => {
            expect(rootElement.getAttribute("data-palette")).toBe("custom");
        });
        await expect(oceanMistRadio).toBeChecked();
        await expect(
            rootElement.style.getPropertyValue("--color-background-rgb").trim(),
        ).toBe(hexToRgbTriplet(backgroundHex));
        await expect(
            rootElement.style.getPropertyValue("--color-text-rgb").trim(),
        ).toBe(hexToRgbTriplet(textHex));
        await expect(
            rootElement.style.getPropertyValue("--color-primary-100-rgb").trim(),
        ).toBe(hexToRgbTriplet(primaryHex));
        await expect(
            rootElement.style.getPropertyValue("--color-secondary-100-rgb").trim(),
        ).toBe(hexToRgbTriplet(secondaryHex));
    },
};

export const RequiresPaletteNameBeforeCreatingCustomPalette: Story = {
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);

        await userEvent.click(
            canvas.getByRole("button", { name: "Create Custom Palette" }),
        );

        await expect(
            canvas.getByText("Palette name is required"),
        ).toBeVisible();
    },
};
