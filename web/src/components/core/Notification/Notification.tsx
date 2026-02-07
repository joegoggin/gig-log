import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";
import styles from "./Notification.module.scss";
import CheckIcon from "@/components/icons/CheckIcon";
import CloseIcon from "@/components/icons/CloseIcon";
import ErrorIcon from "@/components/icons/ErrorIcon";
import InfoIcon from "@/components/icons/InfoIcon";
import WarningIcon from "@/components/icons/WarningIcon";

/**
 * Enum representing the available notification types.
 */
export enum NotificationType {
    /** Informational notification */
    INFO = "info",
    /** Warning notification */
    WARNING = "warning",
    /** Success notification */
    SUCCESS = "success",
    /** Error notification */
    ERROR = "error",
}

/**
 * Props for the Notification component.
 */
export type NotificationProps = {
    /** The type of notification which determines styling and icon */
    type: NotificationType;
    /** The title text displayed in the notification */
    title: string;
    /** The message body displayed in the notification */
    message: string;
    /** Optional callback fired when the notification is dismissed */
    onClose?: () => void;
};

/**
 *
 * A dismissible notification component with animated entrance/exit.
 * Displays contextual feedback messages with appropriate icons and styling
 * based on the notification type.
 *
 * ## Props
 *
 * - `type` - The type of notification which determines styling and icon
 * - `title` - The title text displayed in the notification
 * - `message` - The message body displayed in the notification
 * - `onClose` - Optional callback fired when the notification is dismissed
 *
 * ## Example
 *
 * ```tsx
 * <Notification
 *   type={NotificationType.SUCCESS}
 *   title="Success"
 *   message="Your changes have been saved."
 *   onClose={() => console.log('Notification closed')}
 * />
 * ```
 */
const Notification: React.FC<NotificationProps> = ({
    title,
    type,
    message,
    onClose,
}) => {
    const [showNotification, setShowNotification] = useState<boolean>(true);

    const getClassName = () => {
        let classes = styles["notification"];

        switch (type) {
            case NotificationType.INFO:
                classes += ` ${styles["notification--info"]}`;
                break;
            case NotificationType.WARNING:
                classes += ` ${styles["notification--warning"]}`;
                break;
            case NotificationType.SUCCESS:
                classes += ` ${styles["notification--success"]}`;
                break;
            case NotificationType.ERROR:
                classes += ` ${styles["notification--error"]}`;
                break;
        }

        return classes;
    };

    const getIcon = () => {
        switch (type) {
            case NotificationType.INFO:
                return (
                    <span data-testid="notification-icon-info">
                        <InfoIcon />
                    </span>
                );
            case NotificationType.WARNING:
                return (
                    <span data-testid="notification-icon-warning">
                        <WarningIcon />
                    </span>
                );
            case NotificationType.SUCCESS:
                return (
                    <span data-testid="notification-icon-success">
                        <CheckIcon />
                    </span>
                );
            case NotificationType.ERROR:
                return (
                    <span data-testid="notification-icon-error">
                        <ErrorIcon />
                    </span>
                );
        }
    };

    const handleClose = () => {
        setShowNotification(false);
        onClose?.();
    };

    return (
        <AnimatePresence>
            {showNotification && (
                <motion.div
                    initial={{ scale: 0.95, y: -8 }}
                    animate={{ scale: 1, y: 0 }}
                    exit={{ scale: 0.95, y: -8 }}
                    transition={{ duration: 0.3 }}
                    className={getClassName()}
                    data-testid="notification-root"
                >
                    <div className={styles["notification__icon"]}>
                        {getIcon()}
                    </div>
                    <div className={styles["notification__message"]}>
                        <h5>{title}</h5>
                        <p>{message}</p>
                    </div>
                    <button
                        type="button"
                        className={styles["notification__close"]}
                        onClick={handleClose}
                        aria-label="Close notification"
                    >
                        <CloseIcon />
                    </button>
                </motion.div>
            )}
        </AnimatePresence>
    );
};

export default Notification;
