import {  useEffect, useState } from "react";
import styles from "./RootLayout.module.scss";
import type {ReactNode} from "react";
import type {DeleteModalProps} from "@/components/modals/DeleteModal";
import Notification from "@/components/core/Notification/Notification";
import DeleteModal from "@/components/modals/DeleteModal";
import { useNotification } from "@/contexts/NotificationContext";

/**
 * Props for the RootLayout component.
 */
type RootLayoutProps = {
    /** Additional CSS class names to apply to the layout */
    className?: string;
    /** Whether to render decorative ambient background shapes */
    showAmbient?: boolean;
    /** Content to render inside the layout */
    children: ReactNode;
};

/**
 * Configuration for modals that can be displayed in the layout.
 */
type Modal = {
    /** Configuration for the delete confirmation modal */
    delete?: Omit<DeleteModalProps, "showModal" | "setShowModal">;
};

/**
 *
 * The root layout component that wraps all pages in the application.
 * Handles theme detection, notifications display, and global modals.
 *
 * ## Props
 *
 * - `className` - Additional CSS class names to apply to the layout (default: "")
 * - `showAmbient` - Whether to render decorative background shapes (default: `true`)
 * - `children` - Content to render inside the layout
 *
 * ## Example
 *
 * ```tsx
 * <RootLayout className="home-page">
 *   <HomePage />
 * </RootLayout>
 * ```
 */
function RootLayout({ className = "", showAmbient = true, children }: RootLayoutProps) {
    const [showDeleteModal, setShowDeleteModal] = useState<boolean>(false);
    const { notifications, removeNotification } = useNotification();

    // TODO: Replace with modal state from a modal context/store
    const modal: Modal = {};

    useEffect(() => {
        const prefersDark = window.matchMedia(
            "(prefers-color-scheme: dark)",
        ).matches;
        const theme = prefersDark ? "dark" : "light";

        document.documentElement.setAttribute("data-theme", theme);
    }, []);

    return (
        <>
            <div className={`${styles["root-layout"]} ${className}`}>
                {showAmbient && (
                    <div className={styles["root-layout__ambient"]} aria-hidden="true">
                        <span className={styles["root-layout__orb"]} />
                        <span className={styles["root-layout__orb"]} />
                        <span className={styles["root-layout__orb"]} />
                        <span className={styles["root-layout__orb"]} />
                        <span className={styles["root-layout__orb"]} />
                        <span className={styles["root-layout__orb"]} />
                        <span className={styles["root-layout__orb"]} />
                        <span className={styles["root-layout__orb"]} />
                        <span className={styles["root-layout__orb"]} />
                        <span className={styles["root-layout__orb"]} />
                        <span className={styles["root-layout__orb"]} />
                        <span className={styles["root-layout__orb"]} />
                    </div>
                )}
                <div className={styles["root-layout__notifications"]}>
                    {notifications.map((notification) => (
                        <Notification
                            key={notification.id}
                            {...notification}
                            onClose={() => removeNotification(notification.id)}
                        />
                    ))}
                </div>
                {children}
            </div>
            {showDeleteModal && modal.delete && (
                <DeleteModal
                    showModal={showDeleteModal}
                    setShowModal={setShowDeleteModal}
                    {...modal.delete}
                />
            )}
        </>
    );
}

export default RootLayout;
