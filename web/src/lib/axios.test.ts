/**
 * Unit tests for API client token-refresh behavior.
 *
 * Covered scenarios:
 * - A 401 response triggers `/auth/refresh` and retries the original request.
 * - Auth login failures do not trigger refresh retries.
 *
 * These tests prevent regressions where expired access cookies would force
 * unexpected logouts instead of recovering through refresh-session rotation.
 */
import axios, { AxiosError } from "axios";
import { afterEach, describe, expect, it, vi } from "vitest";
import api from "./axios";
import type { AxiosResponse, InternalAxiosRequestConfig } from "axios";

const originalAdapter = api.defaults.adapter;

afterEach(() => {
    api.defaults.adapter = originalAdapter;
});

const makeResponse = <T>(
    config: InternalAxiosRequestConfig,
    status: number,
    data: T,
): AxiosResponse<T> => ({
    config,
    data,
    status,
    statusText: status === 200 ? "OK" : "Unauthorized",
    headers: {},
});

const createUnauthorizedError = (config: InternalAxiosRequestConfig) => {
    const response = makeResponse(config, 401, { error: "Unauthorized" });
    return new AxiosError(
        "Unauthorized",
        AxiosError.ERR_BAD_REQUEST,
        config,
        undefined,
        response,
    );
};

describe("api axios client", () => {
    it("refreshes and retries once after a 401", async () => {
        let isRefreshed = false;
        const adapter = vi.fn(async (config: InternalAxiosRequestConfig) => {
            if (config.url === "/auth/refresh") {
                isRefreshed = true;
                return makeResponse(config, 200, { message: "ok" });
            }

            if (config.url === "/protected" && !isRefreshed) {
                throw createUnauthorizedError(config);
            }

            if (config.url === "/protected" && isRefreshed) {
                return makeResponse(config, 200, { success: true });
            }

            throw new Error(`unexpected URL: ${config.url}`);
        });

        api.defaults.adapter = adapter;

        const response = await api.get<{ success: boolean }>("/protected");

        expect(response.data.success).toBe(true);
        expect(adapter).toHaveBeenCalledTimes(3);
    });

    it("does not refresh after auth login 401", async () => {
        const adapter = vi.fn(async (config: InternalAxiosRequestConfig) => {
            if (config.url === "/auth/log-in") {
                throw createUnauthorizedError(config);
            }

            if (config.url === "/auth/refresh") {
                return makeResponse(config, 200, { message: "ok" });
            }

            throw new Error(`unexpected URL: ${config.url}`);
        });

        api.defaults.adapter = adapter;

        await expect(
            api.post("/auth/log-in", {
                email: "demo@example.com",
                password: "bad",
            }),
        ).rejects.toBeInstanceOf(axios.AxiosError);

        expect(adapter).toHaveBeenCalledTimes(1);
    });
});
