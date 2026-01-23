import { type ReactNode, type MouseEvent } from "react";
import { useNavigate } from "@tanstack/react-router";
import styles from "./Button.module.scss";

/**
 * Enum representing the available button style variants.
 */
export enum ButtonVariant {
    /** Primary button style with prominent styling */
    PRIMARY,
    /** Secondary button style with subtle styling */
    SECONDARY,
}

/**
 * Props for the Button component.
 */
type ButtonProps = {
    /** Additional CSS class names to apply to the button */
    className?: string;
    /** HTML button type attribute */
    type?: "submit" | "button" | "reset";
    /** URL to navigate to when button is clicked */
    href?: string;
    /** Click event handler */
    onClick?: (e?: MouseEvent<HTMLButtonElement>) => void;
    /** Visual style variant of the button */
    variant?: ButtonVariant;
    /** Content to render inside the button */
    children: ReactNode;
};

/**
 *
 * A reusable button component with support for multiple variants and navigation.
 *
 * ## Props
 *
 * - `className` - Additional CSS class names to apply to the button
 * - `type` - HTML button type attribute (default: "button")
 * - `href` - URL to navigate to when button is clicked
 * - `onClick` - Click event handler
 * - `variant` - Visual style variant of the button (default: PRIMARY)
 * - `children` - Content to render inside the button
 *
 * ## Example
 *
 * ```tsx
 * <Button variant={ButtonVariant.PRIMARY} onClick={handleClick}>
 *   Click Me
 * </Button>
 * ```
 */
function Button({
    className,
    type = "button",
    href,
    onClick,
    variant = ButtonVariant.PRIMARY,
    children,
}: ButtonProps) {
    const navigate = useNavigate();

    const handleClick = (e: MouseEvent<HTMLButtonElement>) => {
        if (type != "submit") {
            e.preventDefault();
        }

        if (onClick) {
            onClick();
        }

        if (href) {
            navigate({ to: href });
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
