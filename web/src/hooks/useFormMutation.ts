import { useMutation } from "@tanstack/react-query";
import type { AxiosError } from "axios";
import type {ValidationErrorResponse} from "@/lib/errors";
import { NotificationType } from "@/components/core/Notification/Notification";
import { useNotification } from "@/contexts/NotificationContext";
import { parseValidationErrors } from "@/lib/errors";

type UseFormMutationOptions<TData, TVariables> = {
    mutationFn: (variables: TVariables) => Promise<TData>;
    onSuccess?: (data: TData) => void;
    onError?: (errors: Record<string, string>) => void;
    fallbackError: string;
};

const useFormMutation = <TData, TVariables = void>({
    mutationFn,
    onSuccess,
    onError,
    fallbackError,
}: UseFormMutationOptions<TData, TVariables>) => {
    const { addNotification } = useNotification();

    return useMutation({
        mutationFn,
        onSuccess,
        onError: (error: AxiosError<ValidationErrorResponse>) => {
            if (error.response?.data.errors) {
                const errors = parseValidationErrors(error.response.data);
                onError?.(errors);
            } else {
                addNotification({
                    type: NotificationType.ERROR,
                    title: "Error",
                    message: fallbackError,
                });
            }
        },
    });
};

export default useFormMutation;
