import { useLocation } from "@tanstack/react-router";
import styles from "./SettingsPage.module.scss";
import Button, { ButtonVariant } from "@/components/core/Button/Button";
import { useAuth } from "@/contexts/AuthContext";
import { settingsSections } from "@/pages/SettingsPage/settingsSections";

/**
 * The authenticated settings hub page.
 * Provides section navigation so users can manage password, email,
 * and appearance preferences on dedicated pages.
 *
 * Route: `/settings`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `Button` - Routes users to each dedicated settings page.
 * - `MainLayout` - Wraps authenticated settings routes.
 */
function SettingsPage() {
    const { pathname } = useLocation();
    const { user } = useAuth();

    return (
        <section className={styles["settings-page"]}>
            <header className={styles["settings-page__hero"]}>
                <p className={styles["settings-page__eyebrow"]}>Account and appearance</p>
                <h1>Settings</h1>
                <p className={styles["settings-page__lead"]}>
                    Manage account security and appearance with focused settings pages.
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

            <article className={styles["settings-page__panel"]}>
                <h2>Choose a settings area</h2>
                <p className={styles["settings-page__panel-lead"]}>
                    Each workflow now has its own route, so you can focus on one type of
                    account update at a time.
                </p>

                <div className={styles["settings-page__cards"]}>
                    {settingsSections
                        .filter((section) => section.href !== "/settings")
                        .map((section) => (
                            <section
                                key={section.href}
                                className={styles["settings-page__card"]}
                            >
                                <h3 className={styles["settings-page__card-title"]}>
                                    {section.label}
                                </h3>
                                <p
                                    className={
                                        styles["settings-page__card-description"]
                                    }
                                >
                                    {section.description}
                                </p>
                                <Button
                                    href={section.href}
                                    variant={ButtonVariant.SECONDARY}
                                    className={styles["settings-page__card-action"]}
                                >
                                    Open {section.label}
                                </Button>
                            </section>
                        ))}
                </div>
            </article>
        </section>
    );
}

export default SettingsPage;
