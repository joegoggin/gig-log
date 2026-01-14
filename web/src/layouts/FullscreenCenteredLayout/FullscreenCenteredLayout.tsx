import RootLayout from "@/layouts/RootLayout/RootLayout";
import type { ReactNode } from "react";
import styles from "./FullscreenCenteredLayout.module.scss";

type FullscreenCenteredLayoutProps = {
    className?: string;
    children: ReactNode;
};

/**
 * Renders a full-screen centered layout by wrapping `children` in RootLayout with the module's centered style merged with any additional classes.
 *
 * @param className - Optional additional CSS class names appended to the component's root element
 * @param children - Content to render inside the layout
 * @returns The rendered layout element with combined class names
 */
function FullscreenCenteredLayout({
    className,
    children,
}: FullscreenCenteredLayoutProps) {
    return (
        <RootLayout
            className={`${styles["fullscreen-centered-layout"]} ${className}`}
        >
            {children}
        </RootLayout>
    );
}

export default FullscreenCenteredLayout;