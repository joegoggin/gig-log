import { useState } from "react";
import type { SetData } from "@/types/SetData";

type FormErrors = Record<string, string>;

const useForm = <T extends Record<string, unknown>>(initialData: T) => {
    const [data, setDataState] = useState<T>(initialData);
    const [errors, setErrors] = useState<FormErrors>({});

    const setData: SetData<T> = (key, value) => {
        setDataState((prev) => ({ ...prev, [key]: value }));
        setErrors({});
    };

    return {
        data,
        errors,
        setData,
        setErrors,
    };
};

export default useForm;
