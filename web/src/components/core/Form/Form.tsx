import { type FormEvent, type ReactNode } from "react";
import styles from "./Form.module.scss";

type FormProps = {
    className?: string;
    onSubmit: () => void;
    children: ReactNode;
};

const Form: React.FC<FormProps> = ({ className = "", onSubmit, children }) => {
    const handleSubmit = (e: FormEvent) => {
        e.preventDefault();

        const allInputs = document.querySelectorAll("input");

        allInputs.forEach((input) => {
            input.blur();
        });

        onSubmit();
    };

    return (
        <form className={`${styles.form} ${className}`} onSubmit={handleSubmit}>
            {children}
        </form>
    );
};

export default Form;
