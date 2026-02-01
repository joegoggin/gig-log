import type { SetData } from "@/types/SetData";
import type { ChangeEvent } from "react";
import { useEffect, useState } from "react";
import styles from "./TextInput.module.scss";

type TextInputProps<T> = {
    className?: string;
    label?: string;
    placeholder?: string;
    name: keyof T;
    data: T;
    setData: SetData<T>;
    password?: boolean;
    errors?: Record<string, string>;
};

const TextInput = <T,>({
    className,
    label,
    placeholder = "",
    name,
    data,
    setData,
    password,
    errors,
}: TextInputProps<T>) => {
    const [classes, setClasses] = useState<string>(styles["text-input"]);
    const [userEditing, setUserEditing] = useState<boolean>(false);
    const [error, setError] = useState<string>();

    const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
        setData(name, e.target.value as T[keyof T]);
        setUserEditing(true);
    };

    useEffect(() => {
        const newError = errors?.[name as string];

        if (newError) {
            setError(newError);
            setUserEditing(false);
        }
    }, [errors, name]);

    useEffect(() => {
        let newClasses = `${styles["text-input"]}`;

        if (error && !userEditing) {
            newClasses += ` ${styles["text-input--error"]}`;
        }

        if (className) {
            newClasses += ` ${className}`;
        }

        setClasses(newClasses);
    }, [error, userEditing, className]);

    return (
        <div className={classes}>
            {label && <label>{label}</label>}
            <input
                type={password ? "password" : "text"}
                placeholder={placeholder}
                onChange={handleChange}
                value={data[name] as string}
            />
            {error && !userEditing && <p>{error}</p>}
        </div>
    );
};

export default TextInput;
