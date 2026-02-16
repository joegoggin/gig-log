import styles from "./HomePage.module.scss";
import RootLayout from "@/layouts/RootLayout/RootLayout";
import Button, { ButtonVariant } from "@/components/core/Button/Button";
import GigLogLogoIcon from "@/components/icons/GigLogLogoIcon";

/**
 * Props for the HomePage component.
 */
type HomePageProps = {
    /** Whether the user is currently authenticated */
    isLoggedIn: boolean;
};

/**
 * The home page and landing page for the application.
 * Displays an introduction to GigLog and provides navigation to
 * sign up, log in, or access the dashboard for authenticated users.
 *
 * Route: `/`
 *
 * ## Props
 *
 * - `isLoggedIn` - Whether the user is currently authenticated
 *
 * ## Related Components
 *
 * - `Button` - Used for navigation actions
 * - `RootLayout` - Global page layout wrapper
 * - `GigLogLogoIcon` - Displays the GigLog brand mark
 */
function HomePage({ isLoggedIn }: HomePageProps) {
    const featureHighlights = [
        {
            title: "One source of truth for every client",
            description:
                "Keep clients, jobs, sessions, and payouts in one dashboard so your records stay organized.",
        },
        {
            title: "Built for freelance cash flow",
            description:
                "Track hourly work and fixed payouts side-by-side so billing details are always clear.",
        },
        {
            title: "Know where your week went",
            description:
                "See your active gigs at a glance and quickly spot what needs attention next.",
        },
    ];

    const workflowSteps = [
        {
            title: "Create your companies",
            description: "Set up your clients once and reuse them for every new job.",
        },
        {
            title: "Log sessions and payouts",
            description: "Capture your work as it happens without juggling multiple tools.",
        },
        {
            title: "Review your dashboard",
            description: "Get a clear snapshot of your freelance business before invoices and tax time.",
        },
    ];

    return (
        <RootLayout className={styles["home-page"]} showAmbient={false}>
            <div className={styles["home-page__ambient"]} aria-hidden="true">
                <span className={styles["home-page__orb"]} />
                <span className={styles["home-page__orb"]} />
                <span className={styles["home-page__orb"]} />
                <span className={styles["home-page__orb"]} />
                <span className={styles["home-page__orb"]} />
                <span className={styles["home-page__orb"]} />
                <span className={styles["home-page__orb"]} />
                <span className={styles["home-page__orb"]} />
                <span className={styles["home-page__orb"]} />
                <span className={styles["home-page__orb"]} />
                <span className={styles["home-page__orb"]} />
                <span className={styles["home-page__orb"]} />
            </div>

            <main className={styles["home-page__content"]}>
                <section className={styles["home-page__hero"]}>
                    <div className={styles["home-page__hero-copy"]}>
                        <p className={styles["home-page__eyebrow"]}>The freelancer command center</p>
                        <div
                            className={styles["home-page__logo"]}
                            aria-label="GigLog"
                            role="img"
                        >
                            <GigLogLogoIcon />
                        </div>
                        <h1>Run every gig with less admin and more clarity.</h1>
                        <p className={styles["home-page__lead"]}>
                            GigLog helps freelancers stay on top of hours, payouts, and client work without
                            maintaining scattered notes and spreadsheets.
                        </p>
                        <div className={styles["home-page__buttons"]}>
                            {isLoggedIn ? (
                                <Button href="/dashboard">View Dashboard</Button>
                            ) : (
                                <>
                                    <Button href="/auth/sign-up">Sign Up</Button>
                                    <Button
                                        href="/auth/log-in"
                                        variant={ButtonVariant.SECONDARY}
                                    >
                                        Log In
                                    </Button>
                                </>
                            )}
                        </div>
                    </div>

                    <aside className={styles["home-page__hero-panel"]}>
                        <p className={styles["home-page__panel-label"]}>Workflow Preview</p>
                        <h2>Everything your freelance business needs in one place.</h2>
                        <ul className={styles["home-page__panel-points"]}>
                            <li>
                                <h3>Client records</h3>
                                <p>Keep company contacts and job details connected from day one.</p>
                            </li>
                            <li>
                                <h3>Work logs</h3>
                                <p>Capture sessions and payouts as you go so billing stays accurate.</p>
                            </li>
                            <li>
                                <h3>Tax-ready history</h3>
                                <p>Review everything by client or date when it is time to file.</p>
                            </li>
                        </ul>
                    </aside>
                </section>

                <section className={styles["home-page__section"]}>
                    <h2>Why freelancers choose GigLog</h2>
                    <div className={styles["home-page__feature-grid"]}>
                        {featureHighlights.map((feature) => (
                            <article
                                key={feature.title}
                                className={styles["home-page__feature-card"]}
                            >
                                <h3>{feature.title}</h3>
                                <p>{feature.description}</p>
                            </article>
                        ))}
                    </div>
                </section>

                <section className={styles["home-page__section"]}>
                    <h2>Simple workflow from first client to payday</h2>
                    <ol className={styles["home-page__steps"]}>
                        {workflowSteps.map((step, index) => (
                            <li key={step.title}>
                                <span>{index + 1}</span>
                                <div>
                                    <h3>{step.title}</h3>
                                    <p>{step.description}</p>
                                </div>
                            </li>
                        ))}
                    </ol>
                </section>
            </main>
        </RootLayout>
    );
}

export default HomePage;
