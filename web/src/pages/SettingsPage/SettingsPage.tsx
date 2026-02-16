
import styles from "./SettingsPage.module.scss";

/**
 * Placeholder page for user and app settings features.
 *
 * Route: `/settings`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `MainLayout` - Wraps the page with primary app navigation.
 */
function SettingsPage() {
    const plannedFeatures = [
        "Profile updates and contact preferences",
        "Default billing and tax preferences",
        "Session and security management options",
    ];

    return (
        <section className={styles["settings-page"]}>
            <header className={styles["settings-page__hero"]}>
                <p className={styles["settings-page__eyebrow"]}>In progress</p>
                <h1>Settings</h1>
                <p className={styles["settings-page__lead"]}>
                    Settings management is coming soon.
                </p>
            </header>
            <div className={styles["settings-page__panel"]}>
                <h2>What is next</h2>
                <ul>
                    {plannedFeatures.map((feature) => (
                        <li key={feature}>{feature}</li>
                    ))}
                </ul>
            </div>
        </section>
    );
}

export default SettingsPage;
