import { useLocation } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import type {
    ColorPalette,
    PaletteRgbTokens,
    ThemeMode,
} from "@/lib/appearance";
import type {
    CreateCustomPaletteRequest,
    CustomPalette,
    UpdateCustomPaletteRequest,
} from "@/types/models/Appearance";
import styles from "@/pages/SettingsPage/SettingsPage.module.scss";
import useForm from "@/hooks/useForm";
import useFormMutation from "@/hooks/useFormMutation";
import Button, { ButtonVariant } from "@/components/core/Button/Button";
import AddIcon from "@/components/icons/AddIcon";
import EditIcon from "@/components/icons/EditIcon";
import Form from "@/components/core/Form/Form";
import { NotificationType } from "@/components/core/Notification/Notification";
import TextInput from "@/components/core/TextInput/TextInput";
import { useAppearance } from "@/contexts/AppearanceContext";
import { useAuth } from "@/contexts/AuthContext";
import { useNotification } from "@/contexts/NotificationContext";
import { settingsSections } from "@/pages/SettingsPage/settingsSections";

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
        value: "catppuccin",
        label: "Catppuccin",
        description:
            "Soothing pastel tones with soft contrast and playful accents.",
    },
    {
        value: "tokyo-night",
        label: "Tokyo Night",
        description:
            "Cool neon-inspired accents balanced for long coding sessions.",
    },
    {
        value: "everforest",
        label: "Everforest",
        description: "Warm, nature-led greens with gentle eye-friendly contrast.",
    },
];

const colorPaletteLabels: Record<ColorPalette, string> = {
    catppuccin: "Catppuccin",
    "tokyo-night": "Tokyo Night",
    everforest: "Everforest",
};

const paletteSeedOptions: Array<PaletteSeedOption> = [
    {
        name: "background_seed_hex",
        label: "Background",
    },
    {
        name: "text_seed_hex",
        label: "Text",
    },
    {
        name: "primary_seed_hex",
        label: "Primary",
    },
    {
        name: "secondary_seed_hex",
        label: "Secondary",
    },
    {
        name: "green_seed_hex",
        label: "Success Accent",
    },
    {
        name: "red_seed_hex",
        label: "Danger Accent",
    },
    {
        name: "yellow_seed_hex",
        label: "Warning Accent",
    },
    {
        name: "blue_seed_hex",
        label: "Info Accent",
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
};

const customPalettePreviewTokens: Array<keyof PaletteRgbTokens> = [
    "background",
    "text",
    "primary_100",
    "secondary_100",
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

const toPaletteFormData = (palette: CustomPalette): UpdateCustomPaletteRequest => {
    return {
        name: palette.name,
        background_seed_hex: palette.background_seed_hex,
        text_seed_hex: palette.text_seed_hex,
        primary_seed_hex: palette.primary_seed_hex,
        secondary_seed_hex: palette.secondary_seed_hex,
        green_seed_hex: palette.green_seed_hex,
        red_seed_hex: palette.red_seed_hex,
        yellow_seed_hex: palette.yellow_seed_hex,
        blue_seed_hex: palette.blue_seed_hex,
        magenta_seed_hex: palette.magenta_seed_hex,
        cyan_seed_hex: palette.cyan_seed_hex,
    };
};

/**
 * The authenticated appearance settings page.
 * Lets users configure persisted theme mode, choose built-in palettes,
 * and create or edit DB-backed custom palettes generated from base seed colors.
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
 * - `Form` - Handles custom palette create/edit submissions.
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
        updateCustomPalette,
    } = useAppearance();
    const [isSelectingPalette, setIsSelectingPalette] = useState(false);
    const [isCreatingPalette, setIsCreatingPalette] = useState(false);
    const [editingPaletteId, setEditingPaletteId] = useState<string | null>(
        null,
    );
    const {
        data: customPaletteData,
        errors: customPaletteErrors,
        setData: setCustomPaletteData,
        setErrors: setCustomPaletteErrors,
    } = useForm<CreateCustomPaletteRequest>(initialPaletteFormData);
    const {
        data: editPaletteData,
        errors: editPaletteErrors,
        setData: setEditPaletteData,
        setErrors: setEditPaletteErrors,
    } = useForm<UpdateCustomPaletteRequest>(initialPaletteFormData);

    const createPaletteMutation = useFormMutation<CustomPalette, void>({
        mutationFn: async () => {
            return createCustomPalette(customPaletteData);
        },
        onSuccess: (palette) => {
            setIsCreatingPalette(false);
            setCustomPaletteData("name", "");
            setCustomPaletteErrors({});
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Palette Created",
                message: `${palette.name} is now active.`,
            });
        },
        onError: setCustomPaletteErrors,
        fallbackError: "Failed to create custom palette",
    });

    const updatePaletteMutation = useFormMutation<CustomPalette, void>({
        mutationFn: async () => {
            if (!editingPaletteId) {
                throw new Error("No custom palette selected for editing.");
            }

            return updateCustomPalette(editingPaletteId, editPaletteData);
        },
        onSuccess: (palette) => {
            setEditingPaletteId(null);
            setEditPaletteErrors({});
            addNotification({
                type: NotificationType.SUCCESS,
                title: "Palette Updated",
                message: `${palette.name} was updated successfully.`,
            });
        },
        onError: setEditPaletteErrors,
        fallbackError: "Failed to update custom palette",
    });

    useEffect(() => {
        if (!editingPaletteId) {
            return;
        }

        const exists = customPalettes.some(
            (palette) => palette.id === editingPaletteId,
        );

        if (!exists) {
            setEditingPaletteId(null);
            setEditPaletteErrors({});
        }
    }, [customPalettes, editingPaletteId, setEditPaletteErrors]);

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

    const handleEditPaletteClick = (palette: CustomPalette) => {
        if (updatePaletteMutation.isPending || createPaletteMutation.isPending) {
            return;
        }

        const formData = toPaletteFormData(palette);

        setIsCreatingPalette(false);
        setCustomPaletteErrors({});
        setEditingPaletteId(palette.id);
        setEditPaletteErrors({});
        setEditPaletteData("name", formData.name);
        setEditPaletteData("background_seed_hex", formData.background_seed_hex);
        setEditPaletteData("text_seed_hex", formData.text_seed_hex);
        setEditPaletteData("primary_seed_hex", formData.primary_seed_hex);
        setEditPaletteData("secondary_seed_hex", formData.secondary_seed_hex);
        setEditPaletteData("green_seed_hex", formData.green_seed_hex);
        setEditPaletteData("red_seed_hex", formData.red_seed_hex);
        setEditPaletteData("yellow_seed_hex", formData.yellow_seed_hex);
        setEditPaletteData("blue_seed_hex", formData.blue_seed_hex);
        setEditPaletteData("magenta_seed_hex", formData.magenta_seed_hex);
        setEditPaletteData("cyan_seed_hex", formData.cyan_seed_hex);
    };

    const handleCancelEditPalette = () => {
        if (updatePaletteMutation.isPending) {
            return;
        }

        setEditingPaletteId(null);
        setEditPaletteErrors({});
    };

    const handleCreatePaletteClick = () => {
        if (createPaletteMutation.isPending || updatePaletteMutation.isPending) {
            return;
        }

        setEditingPaletteId(null);
        setEditPaletteErrors({});
        setCustomPaletteErrors({});
        setIsCreatingPalette(true);
    };

    const handleCancelCreatePalette = () => {
        if (createPaletteMutation.isPending) {
            return;
        }

        setIsCreatingPalette(false);
        setCustomPaletteErrors({});
        setCustomPaletteData("name", "");
    };

    const handleUpdatePaletteSubmit = () => {
        if (!editingPaletteId || updatePaletteMutation.isPending) {
            return;
        }

        if (!editPaletteData.name.trim()) {
            setEditPaletteErrors({
                name: "Palette name is required",
            });
            return;
        }

        updatePaletteMutation.mutate();
    };

    const handleCreatePaletteSubmit = () => {
        if (createPaletteMutation.isPending) {
            return;
        }

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
    const editingPaletteName =
        customPalettes.find((palette) => palette.id === editingPaletteId)?.name ??
        null;

    return (
        <section className={styles["settings-page"]}>
            <header className={styles["settings-page__hero"]}>
                <p className={styles["settings-page__eyebrow"]}>
                    Account and appearance
                </p>
                <h1>Appearance settings</h1>
                <p className={styles["settings-page__lead"]}>
                    Choose how GigLog handles light and dark surfaces, then pick
                    or create a color palette for interface roles and accents.
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
                                                    "settings-page__palette-option-actions"
                                                ]
                                            }
                                        >
                                            <span
                                                className={
                                                    styles[
                                                        "settings-page__palette-option-badge"
                                                    ]
                                                }
                                            >
                                                Custom
                                            </span>
                                            <button
                                                aria-label={`Edit ${palette.name} palette`}
                                                className={`${styles["settings-page__palette-option-edit"]} ${
                                                    editingPaletteId ===
                                                    palette.id
                                                        ? styles[
                                                              "settings-page__palette-option-edit--active"
                                                          ]
                                                        : ""
                                                }`}
                                                onClick={(event) => {
                                                    event.preventDefault();
                                                    event.stopPropagation();
                                                    handleEditPaletteClick(
                                                        palette,
                                                    );
                                                }}
                                                type="button"
                                            >
                                                <EditIcon />
                                                <span>
                                                    {editingPaletteId ===
                                                    palette.id
                                                        ? "Editing"
                                                        : "Edit"}
                                                </span>
                                            </button>
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
                            No custom palettes yet. Use Create to personalize
                            the interface.
                        </p>
                    )}
                </fieldset>

                <p className={styles["settings-page__palette-status"]}>
                    Active palette: <strong>{activePaletteLabel}</strong>.
                </p>

                <div className={styles["settings-page__palette-actions"]}>
                    <button
                        aria-controls="settings-custom-palette-creator"
                        aria-expanded={isCreatingPalette}
                        className={`${styles["settings-page__palette-option-edit"]} ${
                            isCreatingPalette
                                ? styles[
                                      "settings-page__palette-option-edit--active"
                                  ]
                                : ""
                        }`}
                        onClick={handleCreatePaletteClick}
                        type="button"
                    >
                        <AddIcon />
                        <span>Create</span>
                    </button>
                </div>

                {editingPaletteId && editingPaletteName && (
                    <section
                        className={styles["settings-page__custom-palette-editor"]}
                    >
                        <h3>Edit custom palette</h3>
                        <p className={styles["settings-page__step-note"]}>
                            Updating a palette refreshes generated shades while
                            keeping your current active selection.
                        </p>

                        <Form onSubmit={handleUpdatePaletteSubmit}>
                            <TextInput
                                name="name"
                                label="Palette Name"
                                placeholder="Ocean Mist"
                                data={editPaletteData}
                                setData={setEditPaletteData}
                                errors={editPaletteErrors}
                            />

                            <div
                                className={
                                    styles["settings-page__custom-palette-grid"]
                                }
                            >
                                {paletteSeedOptions.map((option) => {
                                    const value = editPaletteData[option.name];
                                    const errorMessage =
                                        editPaletteErrors[option.name];

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
                                                    setEditPaletteData(
                                                        option.name,
                                                        event.currentTarget
                                                            .value,
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

                            <div className={styles["settings-page__form-actions"]}>
                                <Button type="submit">
                                    {updatePaletteMutation.isPending
                                        ? "Saving Changes..."
                                        : "Save Palette Changes"}
                                </Button>
                                <Button
                                    type="button"
                                    variant={ButtonVariant.SECONDARY}
                                    onClick={handleCancelEditPalette}
                                >
                                    Cancel
                                </Button>
                            </div>
                        </Form>
                    </section>
                )}

                {isCreatingPalette && (
                    <section
                        id="settings-custom-palette-creator"
                        className={styles["settings-page__custom-palette-creator"]}
                    >
                        <h3>Create custom palette</h3>
                        <p className={styles["settings-page__step-note"]}>
                            Pick base background/text plus primary/secondary and
                            accent colors. GigLog generates lighter shades
                            automatically.
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
                                                        event.currentTarget
                                                            .value,
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

                            <div
                                className={styles["settings-page__form-actions"]}
                            >
                                <Button type="submit">
                                    {createPaletteMutation.isPending
                                        ? "Creating Palette..."
                                        : "Create Custom Palette"}
                                </Button>
                                <Button
                                    type="button"
                                    variant={ButtonVariant.SECONDARY}
                                    onClick={handleCancelCreatePalette}
                                >
                                    Cancel
                                </Button>
                            </div>
                        </Form>
                    </section>
                )}
            </article>
        </section>
    );
}

export default SettingsAppearancePage;
