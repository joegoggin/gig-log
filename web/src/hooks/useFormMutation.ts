import { useMutation } from "@tanstack/react-query";
import type { AxiosError } from "axios";
import {
    parseValidationErrors,
    type ValidationErrorResponse,
} from "@/lib/errors";

type UseFormMutationOptions<TData, TVariables> = {
    mutationFn: (variables: TVariables) => Promise<TData>;
    onSuccess?: (data: TData) => void;
    onError?: (errors: Record<string, string>) => void;
    fallbackError: { field: string; message: string };
};

const useFormMutation = <TData, TVariables = void>({
    mutationFn,
    onSuccess,
    onError,
    fallbackError,
}: UseFormMutationOptions<TData, TVariables>) => {
    return useMutation({
        mutationFn,
        onSuccess,
        onError: (error: AxiosError<ValidationErrorResponse>) => {
            if (error.response?.data?.errors) {
                const errors = parseValidationErrors(error.response.data);
                onError?.(errors);
            } else {
                onError?.({ [fallbackError.field]: fallbackError.message });
            }
        },
    });
};

export default useFormMutation;
