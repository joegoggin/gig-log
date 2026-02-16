import { useNavigate } from "@tanstack/react-router";
import styles from "./BackButton.module.scss";
import type { ReactNode } from "react";
import BackIcon from "@/components/icons/BackIcon";

/**
 * Props for the BackButton component.
 */
type BackButtonProps = {
    /** URL to navigate to when the button is clicked */
    href: string;
    /** Content to render as the button label */
    children: ReactNode;
};

/**
 * A ghost-style navigation button with a back arrow icon.
 * Visually distinct from action buttons, intended for "go back" navigation.
 * The arrow slides left on hover for a subtle motion cue.
 *
 * ## Props
 *
 * - `href` - URL to navigate to when the button is clicked
 * - `children` - Content to render as the button label
 *
 * ## Example
 *
 * ```tsx
 * <BackButton href="/companies">Back to Companies</BackButton>
 * ```
 */
function BackButton({ href, children }: BackButtonProps) {
    const navigate = useNavigate();

    const handleClick = () => {
        navigate({ to: href });
    };

    return (
        <button type="button" className={styles["back-button"]} onClick={handleClick}>
            <span className={styles["back-button__icon"]}>
                <BackIcon />
            </span>
            <span className={styles["back-button__label"]}>{children}</span>
        </button>
    );
}

export default BackButton;
