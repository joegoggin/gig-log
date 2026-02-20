import { useLocation } from "@tanstack/react-router";
import type { ColorPalette, ThemeMode } from "@/lib/appearance";
import styles from "@/pages/SettingsPage/SettingsPage.module.scss";
import Button, { ButtonVariant } from "@/components/core/Button/Button";
import { useAppearance } from "@/contexts/AppearanceContext";
import { useAuth } from "@/contexts/AuthContext";
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
        description: "Balanced cool accents inspired by the original GigLog look.",
    },
    {
        value: "sunset",
        label: "Sunset",
        description: "Warmer coral and amber accents for a softer contrast profile.",
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

/**
 * The authenticated appearance settings page.
 * Lets users configure persisted theme mode and color palette preferences.
 *
 * Route: `/settings/appearance`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `Button` - Navigates between settings sections.
 * - `AppearanceProvider` - Persists and applies appearance preferences.
 */
function SettingsAppearancePage() {
    const { pathname } = useLocation();
    const { user } = useAuth();
    const { mode, palette, resolvedTheme, setMode, setPalette } = useAppearance();

    return (
        <section className={styles["settings-page"]}>
            <header className={styles["settings-page__hero"]}>
                <p className={styles["settings-page__eyebrow"]}>Account and appearance</p>
                <h1>Appearance settings</h1>
                <p className={styles["settings-page__lead"]}>
                    Choose how GigLog handles light and dark surfaces, then pick an accent
                    palette for the interface.
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
                        pathname === section.href || pathname === `${section.href}/`;

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
                    Preferences are saved automatically and restored when you come back.
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
                                            styles["settings-page__theme-option-label"]
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
                    Active theme: <strong>{resolvedTheme === "dark" ? "Dark" : "Light"}</strong>
                    {mode === "system" ? " (following your device setting)." : "."}
                </p>

                <fieldset className={styles["settings-page__palette-fieldset"]}>
                    <legend className={styles["settings-page__palette-legend"]}>
                        Color palette
                    </legend>

                    <div className={styles["settings-page__palette-options"]}>
                        {colorPaletteOptions.map((option) => {
                            const inputId = `color-palette-${option.value}`;
                            const isActive = palette === option.value;

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
                                        onChange={() => setPalette(option.value)}
                                    />
                                    <span
                                        className={
                                            styles["settings-page__palette-option-label"]
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
                    </div>
                </fieldset>

                <p className={styles["settings-page__palette-status"]}>
                    Active palette: <strong>{colorPaletteLabels[palette]}</strong>.
                </p>
            </article>
        </section>
    );
}

export default SettingsAppearancePage;
