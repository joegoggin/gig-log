
import styles from "./DashboardPage.module.scss";

/**
 * The authenticated dashboard page for signed-in users.
 * Displays dashboard content and provides a log-out action.
 *
 * Route: `/dashboard`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `MainLayout` - Wraps the page with primary app navigation.
 */
function DashboardPage() {
    const quickStats = [
        {
            label: "Client Accounts",
            value: "Companies",
            description: "Review active clients and keep records tidy before billing cycles.",
        },
        {
            label: "Work Tracking",
            value: "Jobs",
            description: "Capture ongoing engagements and know exactly what is in progress.",
        },
        {
            label: "Revenue Flow",
            value: "Payments",
            description: "Track payouts and transfer status to avoid missing incoming cash.",
        },
    ];

    return (
        <section className={styles["dashboard-page"]}>
            <header className={styles["dashboard-page__hero"]}>
                <p className={styles["dashboard-page__eyebrow"]}>Control center</p>
                <h1>Dashboard</h1>
                <p className={styles["dashboard-page__lead"]}>
                    Welcome back. Use the sidebar to navigate across the app.
                </p>
            </header>

            <div className={styles["dashboard-page__grid"]}>
                {quickStats.map((stat) => (
                    <article key={stat.label} className={styles["dashboard-page__card"]}>
                        <p className={styles["dashboard-page__label"]}>{stat.label}</p>
                        <h2>{stat.value}</h2>
                        <p>{stat.description}</p>
                    </article>
                ))}
            </div>
        </section>
    );
}

export default DashboardPage;
