import {  useEffect } from "react";
import { useNavigate } from "@tanstack/react-router";
import styles from "./PrivateRoute.module.scss";
import type {ReactNode} from "react";
import Spinner from "@/components/core/Spinner/Spinner";
import { useAuth } from "@/contexts/AuthContext";

/**
 * Props for the PrivateRoute component.
 */
type PrivateRouteProps = {
    /** Additional CSS class names to apply to the loading state wrapper */
    className?: string;
    /** Content to render when the user is authenticated */
    children: ReactNode;
    /** Route to redirect to when unauthenticated */
    redirectTo?: string;
    /** Label shown while determining authentication status */
    loadingLabel?: string;
    /** Label shown while redirecting to the log in page */
    redirectingLabel?: string;
};

/**
 *
 * A route wrapper that redirects unauthenticated users to the log in page.
 *
 * ## Props
 *
 * - `className` - Additional CSS class names to apply to the loading state wrapper
 * - `children` - Content to render when the user is authenticated
 * - `redirectTo` - Route to redirect to when unauthenticated (default: "/auth/log-in")
 * - `loadingLabel` - Label shown while determining authentication status (default: "Loading")
 * - `redirectingLabel` - Label shown while redirecting to the log in page (default: "Redirecting to log in")
 *
 * ## Example
 *
 * ```tsx
 * <PrivateRoute>
 *   <DashboardPage />
 * </PrivateRoute>
 * ```
 */
function PrivateRoute({
    className = "",
    children,
    redirectTo = "/auth/log-in",
    loadingLabel = "Loading",
    redirectingLabel = "Redirecting to log in",
}: PrivateRouteProps) {
    const { isLoggedIn, isLoading } = useAuth();
    const navigate = useNavigate();
    const wrapperClassName = className
        ? `${styles["private-route"]} ${className}`
        : styles["private-route"];

    useEffect(() => {
        if (!isLoading && !isLoggedIn) {
            navigate({ to: redirectTo });
        }
    }, [isLoading, isLoggedIn, navigate, redirectTo]);

    if (isLoading) {
        return (
            <div className={wrapperClassName}>
                <Spinner label={loadingLabel} />
            </div>
        );
    }

    if (!isLoggedIn) {
        return (
            <div className={wrapperClassName}>
                <Spinner label={redirectingLabel} />
            </div>
        );
    }

    return <>{children}</>;
}

export default PrivateRoute;
