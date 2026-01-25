import type { ReactNode } from "react";
import { useNavigate } from "@tanstack/react-router";
import styles from "./Link.module.scss";

type LinkProps = {
    href: string;
    className?: string;
    children: ReactNode;
};

function Link({ href, className, children }: LinkProps) {
    const navigate = useNavigate();

    const handleClick = () => {
        navigate({ to: href });
    };

    const getClassName = () => {
        let classes = styles["link"];

        if (className) {
            classes += ` ${className}`;
        }

        return classes;
    };

    return (
        <span className={getClassName()} onClick={handleClick} role="button">
            {children}
        </span>
    );
}

export default Link;
