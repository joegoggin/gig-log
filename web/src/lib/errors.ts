export type FieldError = {
    field: string;
    message: string;
};

export type ValidationErrorResponse = {
    errors: FieldError[];
};

export function parseValidationErrors(
    response: ValidationErrorResponse,
): Record<string, string> {
    const errors: Record<string, string> = {};

    for (const error of response.errors) {
        errors[error.field] = error.message;
    }

    return errors;
}
