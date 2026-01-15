import Button, { ButtonVariant } from "@/components/core/Button/Button";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import { createFileRoute } from "@tanstack/react-router";
import axios from "axios";
import styles from "./HomePage.module.scss";

/**
 * API response type for the hello endpoint.
 */
type HelloResponse = {
    /** The welcome message from the API */
    message: string;
};

export const Route = createFileRoute("/")({
    component: HomePage,
    loader: async () => {
        try {
            const { data } = await axios.get<HelloResponse>(
                "http://localhost:8000/",
            );

            return data;
        } catch (error) {
            return { message: "failed to fetch from api" };
        }
    },
});

/**
 * The home page and landing page for the application.
 * Displays an introduction to GigLog and provides navigation to
 * sign up, log in, or access the dashboard for authenticated users.
 *
 * Route: `/`
 *
 * Loader Data:
 * - `message` - The welcome message from the API
 */
function HomePage() {
    const isLoggedIn = false;

    return (
        <FullscreenCenteredLayout className={styles["home-page"]}>
            <h1>GigLog</h1>
            <div className={styles["home-page__text"]}>
                <p>
                    Freelancing opens a whole new world of opportunities giving
                    you the freedom to work on your own terms. This freedom
                    comes at a cost though. As a freelancer you are expected to
                    keep track of your own hours, payments, taxes, and expenses.
                    This can be hard to keep track of especially when you are
                    working multiple different gigs as many freelancers do. Our
                    goal at GigLog is to create a robust platform that has
                    everything you need to keep track of in one place.
                </p>
            </div>
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
        </FullscreenCenteredLayout>
    );
}
