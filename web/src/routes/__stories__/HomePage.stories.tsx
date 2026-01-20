import type { Meta, StoryObj } from "@storybook/react-vite";
import Button, { ButtonVariant } from "@/components/core/Button/Button";
import FullscreenCenteredLayout from "@/layouts/FullscreenCenteredLayout/FullscreenCenteredLayout";
import styles from "../HomePage.module.scss";

/**
 * The HomePage component extracted for Storybook documentation.
 * This mirrors the actual HomePage route component.
 */
type HomePageProps = {
    /** Whether the user is logged in */
    isLoggedIn?: boolean;
};

const HomePage = ({ isLoggedIn = false }: HomePageProps) => {
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
};

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
const meta: Meta<typeof HomePage> = {
    title: "Routes/HomePage",
    component: HomePage,
    tags: ["autodocs"],
    parameters: {
        layout: "fullscreen",
    },
    argTypes: {
        isLoggedIn: {
            control: { type: "boolean" },
            description: "Whether the user is logged in",
        },
    },
};

export default meta;
type Story = StoryObj<typeof HomePage>;

export const LoggedOut: Story = {
    args: {
        isLoggedIn: false,
    },
};

export const LoggedIn: Story = {
    args: {
        isLoggedIn: true,
    },
};
