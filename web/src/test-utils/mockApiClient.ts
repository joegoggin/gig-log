/**
 * Test helpers for deterministic API mocking across Storybook and unit tests.
 *
 * Covered behavior:
 * - Creates axios-shaped success/error responses for form mutation flows.
 * - Provides temporary `api.post` / `api.get` replacements with restore hooks.
 * - Standardizes validation-error payloads used by field error assertions.
 */
import type { AxiosError, AxiosResponse, InternalAxiosRequestConfig } from "axios";
import type { ValidationErrorResponse } from "@/lib/errors";
import api from "@/lib/axios";

const createRequestConfig = (): InternalAxiosRequestConfig =>
    ({
        headers: {},
    }) as InternalAxiosRequestConfig;

export function createMockApiResponse<T>(
    data: T,
    status = 200,
    statusText = "OK",
): AxiosResponse<T> {
    return {
        data,
        status,
        statusText,
        headers: {},
        config: createRequestConfig(),
    };
}

export function createValidationAxiosError(
    errors: ValidationErrorResponse["errors"],
): AxiosError<ValidationErrorResponse> {
    return {
        name: "AxiosError",
        message: "Validation failed",
        isAxiosError: true,
        toJSON: () => ({}),
        config: createRequestConfig(),
        response: {
            data: { errors },
            status: 422,
            statusText: "Unprocessable Entity",
            headers: {},
            config: createRequestConfig(),
        },
    } as AxiosError<ValidationErrorResponse>;
}

export function createAxiosErrorResponse<T>(
    data: T,
    status = 500,
    statusText = "Error",
): AxiosError<T> {
    return {
        name: "AxiosError",
        message: statusText,
        isAxiosError: true,
        toJSON: () => ({}),
        config: createRequestConfig(),
        response: {
            data,
            status,
            statusText,
            headers: {},
            config: createRequestConfig(),
        },
    } as AxiosError<T>;
}

export function mockApiPost(implementation: typeof api.post): () => void {
    const originalPost = api.post;
    api.post = implementation;

    return () => {
        api.post = originalPost;
    };
}

export function mockApiGet(implementation: typeof api.get): () => void {
    const originalGet = api.get;
    api.get = implementation;

    return () => {
        api.get = originalGet;
    };
}

export function mockApiPut(implementation: typeof api.put): () => void {
    const originalPut = api.put;
    api.put = implementation;

    return () => {
        api.put = originalPut;
    };
}

export function mockApiDelete(implementation: typeof api.delete): () => void {
    const originalDelete = api.delete;
    api.delete = implementation;

    return () => {
        api.delete = originalDelete;
    };
}

export function mockApiPostHandler(
    handler: (
        url: string,
        data?: unknown,
        config?: unknown,
    ) => Promise<AxiosResponse<unknown>>,
): () => void {
    return mockApiPost(((url, data, config) =>
        handler(url, data, config)) as typeof api.post);
}

export function mockApiGetHandler(
    handler: (
        url: string,
        config?: unknown,
    ) => Promise<AxiosResponse<unknown>>,
): () => void {
    return mockApiGet(((url, config) => handler(url, config)) as typeof api.get);
}

export function mockApiPutHandler(
    handler: (
        url: string,
        data?: unknown,
        config?: unknown,
    ) => Promise<AxiosResponse<unknown>>,
): () => void {
    return mockApiPut(((url, data, config) =>
        handler(url, data, config)) as typeof api.put);
}

export function mockApiDeleteHandler(
    handler: (url: string, config?: unknown) => Promise<AxiosResponse<unknown>>,
): () => void {
    return mockApiDelete(((url, config) => handler(url, config)) as typeof api.delete);
}
