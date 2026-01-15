import CheckIcon from "@/components/icons/CheckIcon";
import CloseIcon from "@/components/icons/CloseIcon";
import ErrorIcon from "@/components/icons/ErrorIcon";
import InfoIcon from "@/components/icons/InfoIcon";
import WarningIcon from "@/components/icons/WarningIcon";
import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";
import styles from "./Notification.module.scss";

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
};

/**
 * A dismissible notification component with animated entrance/exit.
 * Displays contextual feedback messages with appropriate icons and styling
 * based on the notification type.
 *
 * Props:
 * - `type` - The type of notification which determines styling and icon
 * - `title` - The title text displayed in the notification
 * - `message` - The message body displayed in the notification
 *
 * @example
 * ```tsx
 * <Notification
 *   type={NotificationType.SUCCESS}
 *   title="Success"
 *   message="Your changes have been saved."
 * />
 * ```
 */
const Notification: React.FC<NotificationProps> = ({
    title,
    type,
    message,
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
                return <InfoIcon />;
            case NotificationType.WARNING:
                return <WarningIcon />;
            case NotificationType.SUCCESS:
                return <CheckIcon />;
            case NotificationType.ERROR:
                return <ErrorIcon />;
        }
    };

    const handleClose = () => {
        setShowNotification(false);
    };

    return (
        <AnimatePresence>
            {showNotification && (
                <motion.div
                    initial={{ opacity: 0, scale: 0 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0, scale: 0 }}
                    transition={{ duration: 0.3 }}
                    className={getClassName()}
                >
                    <div className={styles["notification__icon"]}>
                        {getIcon()}
                    </div>
                    <div className={styles["notification__message"]}>
                        <h5>{title}</h5>
                        <p>{message}</p>
                    </div>
                    <div
                        className={styles["notification__close"]}
                        onClick={handleClose}
                        role="button"
                    >
                        <CloseIcon />
                    </div>
                </motion.div>
            )}
        </AnimatePresence>
    );
};

export default Notification;
