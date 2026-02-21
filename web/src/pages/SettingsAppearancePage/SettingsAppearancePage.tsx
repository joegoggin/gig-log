import { useLocation } from "@tanstack/react-router";
import { useState } from "react";
import type {
    ColorPalette,
    PaletteRgbTokens,
    ThemeMode,
} from "@/lib/appearance";
import styles from "@/pages/SettingsPage/SettingsPage.module.scss";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import Button, { ButtonVariant } from "@/components/core/Button/Button";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import TextInput from "@/components/core/TextInput/TextInput";
import { useAppearance } from "@/contexts/AppearanceContext";
import { useAuth } from "@/contexts/AuthContext";
import { useNotification } from "@/contexts/NotificationContext";
import { settingsSections } from "@/pages/SettingsPage/settingsSections";
import type {
    CreateCustomPaletteRequest,
    CustomPalette,
} from "@/types/models/Appearance";

type ThemeModeOption = {
    /** Theme mode value applied when selected */
    value: ThemeMode;
    /** Visible label for the mode option */
    label: string;
    /** Supporting copy describing the mode behavior */
    description: string;
};

type ColorPaletteOption = {
    /** Palette value applied when selected */
    value: ColorPalette;
    /** Visible label for the palette option */
    label: string;
    /** Supporting copy describing the palette style */
    description: string;
};

type PaletteSeedName = Exclude<keyof CreateCustomPaletteRequest, "name">;

type PaletteSeedOption = {
    /** Seed field key used in create-palette payloads */
    name: PaletteSeedName;
    /** Visible label shown beside the color picker */
    label: string;
};

const themeModeOptions: Array<ThemeModeOption> = [
    {
        value: "system",
        label: "System",
        description: "Automatically matches your device preference.",
    },
    {
        value: "light",
        label: "Light",
        description: "Keeps surfaces bright for daytime readability.",
    },
    {
        value: "dark",
        label: "Dark",
        description: "Uses darker surfaces for lower-glare sessions.",
    },
];

const colorPaletteOptions: Array<ColorPaletteOption> = [
    {
        value: "default",
        label: "Default",
        description:
            "Balanced cool accents inspired by the original GigLog look.",
    },
    {
        value: "sunset",
        label: "Sunset",
        description:
            "Warmer coral and amber accents for a softer contrast profile.",
    },
    {
        value: "forest",
        label: "Forest",
        description: "Earthy greens and teals for a grounded, natural feel.",
    },
];

const colorPaletteLabels: Record<ColorPalette, string> = {
    default: "Default",
    sunset: "Sunset",
    forest: "Forest",
};

const paletteSeedOptions: Array<PaletteSeedOption> = [
    {
        name: "green_seed_hex",
        label: "Green Accent",
    },
    {
        name: "red_seed_hex",
        label: "Red Accent",
    },
    {
        name: "yellow_seed_hex",
        label: "Yellow Accent",
    },
    {
        name: "blue_seed_hex",
        label: "Blue Accent",
    },
    {
        name: "magenta_seed_hex",
        label: "Magenta Accent",
    },
    {
        name: "cyan_seed_hex",
        label: "Cyan Accent",
    },
];

const initialPaletteFormData: CreateCustomPaletteRequest = {
    name: "",
    green_seed_hex: "#66bb6a",
    red_seed_hex: "#e27d7c",
    yellow_seed_hex: "#d0a761",
    blue_seed_hex: "#5c93cd",
    magenta_seed_hex: "#a082ce",
    cyan_seed_hex: "#59b7aa",
};

const customPalettePreviewTokens: Array<keyof PaletteRgbTokens> = [
    "green_100",
    "red_100",
    "yellow_100",
    "blue_100",
    "magenta_100",
    "cyan_100",
];

const toRgbColor = (rgbTriplet: string): string => {
    return `rgb(${rgbTriplet})`;
};

/**
 * The authenticated appearance settings page.
 * Lets users configure persisted theme mode, choose built-in palettes,
 * and create DB-backed custom palettes generated from base seed colors.
 *
 * Route: `/settings/appearance`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `Button` - Navigates settings sections and submits palette actions.
 * - `Form` - Handles custom palette creation submission.
 * - `TextInput` - Captures custom palette naming input.
 * - `AppearanceProvider` - Syncs palette preferences and custom palettes.
 */
function SettingsAppearancePage() {
    const { pathname } = useLocation();
    const { user } = useAuth();
    const { addNotification } = useNotification();
    const {
        mode,
        activePalette,
        customPalettes,
        resolvedTheme,
        setMode,
        selectPresetPalette,
        selectCustomPalette,
        createCustomPalette,
    } = useAppearance();
    const [isSelectingPalette, setIsSelectingPalette] = useState(false);
    const {
        data: customPaletteData,
        errors: customPaletteErrors,
        setData: setCustomPaletteData,
        setErrors: setCustomPaletteErrors,
    } = useForm<CreateCustomPaletteRequest>(initialPaletteFormData);

    const createPaletteMutation = useFormMutation<CustomPalette, void>({
        mutationFn: async () => {
            return createCustomPalette(customPaletteData);
        },
        onSuccess: (palette) => {
            setCustomPaletteData("name", "");
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Palette Created",
                message: `${palette.name} is now active.`,
            });
        },
        onError: setCustomPaletteErrors,
        fallbackError: "Failed to create custom palette",
    });

    const handlePresetPaletteSelection = (palette: ColorPalette) => {
        if (isSelectingPalette) {
            return;
        }

        setIsSelectingPalette(true);
        void selectPresetPalette(palette)
            .catch(() => {
                addNotification({
                    type: NotificationType.ERROR,
                    title: "Palette Update Failed",
                    message: "Unable to update your active palette right now.",
                });
            })
            .finally(() => {
                setIsSelectingPalette(false);
            });
    };

    const handleCustomPaletteSelection = (customPaletteId: string) => {
        if (isSelectingPalette) {
            return;
        }

        setIsSelectingPalette(true);
        void selectCustomPalette(customPaletteId)
            .catch(() => {
                addNotification({
                    type: NotificationType.ERROR,
                    title: "Palette Update Failed",
                    message: "Unable to activate this custom palette.",
                });
            })
            .finally(() => {
                setIsSelectingPalette(false);
            });
    };

    const handleCreatePaletteSubmit = () => {
        if (!customPaletteData.name.trim()) {
            setCustomPaletteErrors({
                name: "Palette name is required",
            });
            return;
        }

        createPaletteMutation.mutate();
    };

    const activePaletteLabel =
        activePalette.palette_type === "preset" && activePalette.preset_palette
            ? colorPaletteLabels[activePalette.preset_palette]
            : (customPalettes.find(
                  (palette) => palette.id === activePalette.custom_palette_id,
              )?.name ?? "Custom");

    return (
        <section className={styles["settings-page"]}>
            <header className={styles["settings-page__hero"]}>
                <p className={styles["settings-page__eyebrow"]}>
                    Account and appearance
                </p>
                <h1>Appearance settings</h1>
                <p className={styles["settings-page__lead"]}>
                    Choose how GigLog handles light and dark surfaces, then pick
                    or create a color palette for the interface.
                </p>
                {user?.email && (
                    <p className={styles["settings-page__current-email"]}>
                        Signed in as <strong>{user.email}</strong>
                    </p>
                )}
            </header>

            <nav
                className={styles["settings-page__subnav"]}
                aria-label="Settings sections"
            >
                {settingsSections.map((section) => {
                    const isActive =
                        pathname === section.href ||
                        pathname === `${section.href}/`;

                    return (
                        <Button
                            key={section.href}
                            href={section.href}
                            variant={
                                isActive
                                    ? ButtonVariant.PRIMARY
                                    : ButtonVariant.SECONDARY
                            }
                            className={styles["settings-page__subnav-button"]}
                        >
                            {section.label}
                        </Button>
                    );
                })}
            </nav>

            <article
                className={`${styles["settings-page__panel"]} ${styles["settings-page__panel--appearance"]}`}
            >
                <h2>Theme and Palette</h2>
                <p className={styles["settings-page__panel-lead"]}>
                    Theme mode and active palette are restored automatically on
                    your account.
                </p>

                <fieldset className={styles["settings-page__theme-fieldset"]}>
                    <legend className={styles["settings-page__theme-legend"]}>
                        Appearance mode
                    </legend>

                    <div className={styles["settings-page__theme-options"]}>
                        {themeModeOptions.map((option) => {
                            const inputId = `theme-mode-${option.value}`;
                            const isActive = mode === option.value;

                            return (
                                <label
                                    key={option.value}
                                    htmlFor={inputId}
                                    className={`${styles["settings-page__theme-option"]} ${
                                        isActive
                                            ? styles[
                                                  "settings-page__theme-option--active"
                                              ]
                                            : ""
                                    }`}
                                >
                                    <input
                                        id={inputId}
                                        type="radio"
                                        name="theme-mode"
                                        value={option.value}
                                        checked={isActive}
                                        onChange={() => setMode(option.value)}
                                    />
                                    <span
                                        className={
                                            styles[
                                                "settings-page__theme-option-label"
                                            ]
                                        }
                                    >
                                        {option.label}
                                    </span>
                                    <span
                                        className={
                                            styles[
                                                "settings-page__theme-option-description"
                                            ]
                                        }
                                    >
                                        {option.description}
                                    </span>
                                </label>
                            );
                        })}
                    </div>
                </fieldset>

                <p className={styles["settings-page__theme-status"]}>
                    Active theme:{" "}
                    <strong>
                        {resolvedTheme === "dark" ? "Dark" : "Light"}
                    </strong>
                    {mode === "system"
                        ? " (following your device setting)."
                        : "."}
                </p>

                <fieldset className={styles["settings-page__palette-fieldset"]}>
                    <legend className={styles["settings-page__palette-legend"]}>
                        Color palette
                    </legend>

                    <div className={styles["settings-page__palette-options"]}>
                        {colorPaletteOptions.map((option) => {
                            const inputId = `color-palette-${option.value}`;
                            const isActive =
                                activePalette.palette_type === "preset" &&
                                activePalette.preset_palette === option.value;

                            return (
                                <label
                                    key={option.value}
                                    htmlFor={inputId}
                                    className={`${styles["settings-page__palette-option"]} ${
                                        isActive
                                            ? styles[
                                                  "settings-page__palette-option--active"
                                              ]
                                            : ""
                                    }`}
                                >
                                    <input
                                        id={inputId}
                                        type="radio"
                                        name="color-palette"
                                        value={option.value}
                                        checked={isActive}
                                        onChange={() =>
                                            handlePresetPaletteSelection(
                                                option.value,
                                            )
                                        }
                                    />
                                    <span
                                        className={
                                            styles[
                                                "settings-page__palette-option-label"
                                            ]
                                        }
                                    >
                                        {option.label}
                                    </span>
                                    <span
                                        className={
                                            styles[
                                                "settings-page__palette-option-description"
                                            ]
                                        }
                                    >
                                        {option.description}
                                    </span>
                                </label>
                            );
                        })}

                        {customPalettes.map((palette) => {
                            const inputId = `custom-palette-${palette.id}`;
                            const isActive =
                                activePalette.palette_type === "custom" &&
                                activePalette.custom_palette_id === palette.id;

                            return (
                                <label
                                    key={palette.id}
                                    htmlFor={inputId}
                                    className={`${styles["settings-page__palette-option"]} ${
                                        isActive
                                            ? styles[
                                                  "settings-page__palette-option--active"
                                              ]
                                            : ""
                                    }`}
                                >
                                    <input
                                        id={inputId}
                                        type="radio"
                                        name="color-palette"
                                        value={palette.id}
                                        checked={isActive}
                                        onChange={() =>
                                            handleCustomPaletteSelection(
                                                palette.id,
                                            )
                                        }
                                    />
                                    <span
                                        className={
                                            styles[
                                                "settings-page__palette-option-header"
                                            ]
                                        }
                                    >
                                        <span
                                            className={
                                                styles[
                                                    "settings-page__palette-option-label"
                                                ]
                                            }
                                        >
                                            {palette.name}
                                        </span>
                                        <span
                                            className={
                                                styles[
                                                    "settings-page__palette-option-badge"
                                                ]
                                            }
                                        >
                                            Custom
                                        </span>
                                    </span>
                                    <span
                                        className={
                                            styles[
                                                "settings-page__palette-option-description"
                                            ]
                                        }
                                    >
                                        Generated shades from your base accent
                                        colors.
                                    </span>
                                    <span
                                        className={
                                            styles[
                                                "settings-page__palette-swatches"
                                            ]
                                        }
                                        aria-hidden="true"
                                    >
                                        {customPalettePreviewTokens.map(
                                            (token) => (
                                                <span
                                                    key={token}
                                                    className={
                                                        styles[
                                                            "settings-page__palette-swatch"
                                                        ]
                                                    }
                                                    style={{
                                                        backgroundColor:
                                                            toRgbColor(
                                                                palette
                                                                    .generated_tokens[
                                                                    token
                                                                ],
                                                            ),
                                                    }}
                                                />
                                            ),
                                        )}
                                    </span>
                                </label>
                            );
                        })}
                    </div>

                    {customPalettes.length === 0 && (
                        <p
                            className={
                                styles["settings-page__custom-palette-empty"]
                            }
                        >
                            No custom palettes yet. Create one below to
                            personalize the interface.
                        </p>
                    )}
                </fieldset>

                <p className={styles["settings-page__palette-status"]}>
                    Active palette: <strong>{activePaletteLabel}</strong>.
                </p>

                <section
                    className={styles["settings-page__custom-palette-creator"]}
                >
                    <h3>Create custom palette</h3>
                    <p className={styles["settings-page__step-note"]}>
                        Pick six base accents and GigLog generates lighter
                        shades automatically.
                    </p>

                    <Form onSubmit={handleCreatePaletteSubmit}>
                        <TextInput
                            name="name"
                            label="Palette Name"
                            placeholder="Ocean Mist"
                            data={customPaletteData}
                            setData={setCustomPaletteData}
                            errors={customPaletteErrors}
                        />

                        <div
                            className={
                                styles["settings-page__custom-palette-grid"]
                            }
                        >
                            {paletteSeedOptions.map((option) => {
                                const value = customPaletteData[option.name];
                                const errorMessage =
                                    customPaletteErrors[option.name];

                                return (
                                    <label
                                        key={option.name}
                                        className={
                                            styles[
                                                "settings-page__custom-palette-color"
                                            ]
                                        }
                                    >
                                        <span>{option.label}</span>
                                        <input
                                            className={
                                                styles[
                                                    "settings-page__custom-palette-color-input"
                                                ]
                                            }
                                            type="color"
                                            value={value}
                                            onChange={(event) =>
                                                setCustomPaletteData(
                                                    option.name,
                                                    event.currentTarget.value,
                                                )
                                            }
                                        />
                                        <span
                                            className={
                                                styles[
                                                    "settings-page__custom-palette-color-value"
                                                ]
                                            }
                                        >
                                            {value.toUpperCase()}
                                        </span>
                                        {errorMessage && (
                                            <p
                                                className={
                                                    styles[
                                                        "settings-page__field-error"
                                                    ]
                                                }
                                            >
                                                {errorMessage}
                                            </p>
                                        )}
                                    </label>
                                );
                            })}
                        </div>

                        <Button type="submit">
                            {createPaletteMutation.isPending
                                ? "Creating Palette..."
                                : "Create Custom Palette"}
                        </Button>
                    </Form>
                </section>
            </article>
        </section>
    );
}

export default SettingsAppearancePage;
