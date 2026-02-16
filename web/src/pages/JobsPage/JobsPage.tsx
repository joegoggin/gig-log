
import styles from "./JobsPage.module.scss";

/**
 * Placeholder page for job tracking features.
 *
 * Route: `/jobs`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `MainLayout` - Wraps the page with primary app navigation.
 */
function JobsPage() {
    const plannedFeatures = [
        "Create and organize jobs by client",
        "Track hourly and fixed work details",
        "Review status at a glance before invoicing",
    ];

    return (
        <section className={styles["jobs-page"]}>
            <header className={styles["jobs-page__hero"]}>
                <p className={styles["jobs-page__eyebrow"]}>In progress</p>
                <h1>Jobs</h1>
                <p className={styles["jobs-page__lead"]}>Job tracking is coming soon.</p>
            </header>
            <div className={styles["jobs-page__panel"]}>
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

export default JobsPage;
