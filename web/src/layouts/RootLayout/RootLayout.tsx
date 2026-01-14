import { useEffect, useState, type ReactNode } from "react";
import Notification, {
    type NotificationProps,
} from "@/components/core/Notification/Notification";
import DeleteModal, {
    type DeleteModalProps,
} from "@/components/modals/DeleteModal";
import styles from "./RootLayout.module.scss";

type RootLayoutProps = {
    className?: string;
    children: ReactNode;
};

type Modal = {
    delete?: Omit<DeleteModalProps, "showModal" | "setShowModal">;
};

/**
 * Renders the application's root layout, applies the user's preferred color scheme to the document, and provides slots for notifications, children content, and an optional delete confirmation modal.
 *
 * @param className - Additional CSS class names to apply to the root container
 * @param children - Child nodes to render inside the layout
 * @returns The rendered root layout element
 */
function RootLayout({ className = "", children }: RootLayoutProps) {
    const [showDeleteModal, setShowDeleteModal] = useState<boolean>(false);

    const notifications: NotificationProps[] = [];

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
                <div className={styles["root-layout__notifications"]}>
                    {notifications?.map((props, index) => (
                        <Notification key={index} {...props} />
                    ))}
                </div>
                {children}
            </div>
            {showDeleteModal && modal?.delete && (
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