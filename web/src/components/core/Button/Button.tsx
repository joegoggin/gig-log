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
