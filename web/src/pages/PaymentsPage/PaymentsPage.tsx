
import styles from "./PaymentsPage.module.scss";

/**
 * Placeholder page for payment tracking features.
 *
 * Route: `/payments`
 *
 * ## Props
 *
 * - None.
 *
 * ## Related Components
 *
 * - `MainLayout` - Wraps the page with primary app navigation.
 */
function PaymentsPage() {
    const plannedFeatures = [
        "Record incoming payouts by client",
        "Track payment and transfer status in one place",
        "Review totals by date range for tax prep",
    ];

    return (
        <section className={styles["payments-page"]}>
            <header className={styles["payments-page__hero"]}>
                <p className={styles["payments-page__eyebrow"]}>In progress</p>
                <h1>Payments</h1>
                <p className={styles["payments-page__lead"]}>Payment tracking is coming soon.</p>
            </header>
            <div className={styles["payments-page__panel"]}>
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

export default PaymentsPage;
