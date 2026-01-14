import RootLayout from "@/layouts/RootLayout/RootLayout";
import type { ReactNode } from "react";
import styles from "./FullscreenCenteredLayout.module.scss";

type FullscreenCenteredLayoutProps = {
    className?: string;
    children: ReactNode;
};

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
