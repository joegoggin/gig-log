import RootLayout from "@/layouts/RootLayout/RootLayout";
import type { ReactNode } from "react";
import styles from "./FullscreenCenteredLayout.module.scss";

/**
 * Props for the FullscreenCenteredLayout component.
 */
type FullscreenCenteredLayoutProps = {
    /** Additional CSS class names to apply to the layout */
    className?: string;
    /** Content to render inside the layout */
    children: ReactNode;
};

/**
 *
 * A layout component that wraps content in a fullscreen container with centered content.
 * Extends the RootLayout with additional centering styles.
 *
 * ## Props
 *
 * - `className` - Additional CSS class names to apply to the layout
 * - `children` - Content to render inside the layout
 *
 * ## Example
 *
 * ```tsx
 * <FullscreenCenteredLayout className="login-page">
 *   <LoginForm />
 * </FullscreenCenteredLayout>
 * ```
 */
function FullscreenCenteredLayout({
    className,
    children,
}: FullscreenCenteredLayoutProps) {
    return (
        <RootLayout
            className={`${styles["fullscreen-centered-layout"]}${className ? ` ${className}` : ""}`}
        >
            {children}
        </RootLayout>
    );
}

export default FullscreenCenteredLayout;
