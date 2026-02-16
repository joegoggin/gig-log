import styles from "./Form.module.scss";
import type {FormEvent, ReactNode} from "react";

type FormProps = {
    className?: string;
    onSubmit: () => void;
    children: ReactNode;
};

const Form: React.FC<FormProps> = ({ className = "", onSubmit, children }) => {
    const handleSubmit = (e: FormEvent<HTMLFormElement>) => {
        e.preventDefault();

        const formInputs = e.currentTarget.querySelectorAll("input");

        formInputs.forEach((input) => {
            input.blur();
        });

        onSubmit();
    };

    return (
        <form
            className={`${styles.form} ${className}`}
            onSubmit={handleSubmit}
            noValidate
        >
            {children}
        </form>
    );
};

export default Form;
