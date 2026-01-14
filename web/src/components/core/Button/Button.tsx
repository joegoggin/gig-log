import { type ReactNode, type MouseEvent } from "react";
import styles from "./Button.module.scss";

export enum ButtonVariant {
    PRIMARY,
    SECONDARY,
}

type ButtonProps = {
    className?: string;
    type?: "submit" | "button" | "reset";
    href?: string;
    onClick?: (e?: any) => void;
    variant?: ButtonVariant;
    children: ReactNode;
};

/**
 * Renders a styled button element that applies a variant-specific CSS class and forwards click interactions.
 *
 * @param className - Additional CSS class names to append to the button's class list
 * @param type - Button type attribute (e.g., "button", "submit", "reset")
 * @param href - Optional URL associated with the button; this component does not perform navigation
 * @param onClick - Optional callback invoked when the button is clicked; called without arguments
 * @param variant - Visual variant of the button which determines which CSS modifier class is applied
 * @param children - Content to render inside the button
 * @returns The rendered button element with configured classes, type, click behavior, and children
 */
function Button({
    className,
    type = "button",
    href,
    onClick,
    variant = ButtonVariant.PRIMARY,
    children,
}: ButtonProps) {
    const handleClick = (e: MouseEvent<HTMLButtonElement>) => {
        if (type != "submit") {
            e.preventDefault();
        }

        if (onClick) {
            onClick();
        }

        if (href) {
            // router.get(href);
        }
    };

    const getClassName = () => {
        let classes = styles["button"];

        switch (variant) {
            case ButtonVariant.PRIMARY:
                classes += ` ${styles["button--primary"]}`;
                break;
            case ButtonVariant.SECONDARY:
                classes += ` ${styles["button--secondary"]}`;
        }

        if (className) {
            classes += ` ${className}`;
        }

        return classes;
    };

    return (
        <button type={type} className={getClassName()} onClick={handleClick}>
            {children}
        </button>
    );
}

export default Button;