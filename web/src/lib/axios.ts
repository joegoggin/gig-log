import axios from "axios";
import type { InternalAxiosRequestConfig } from "axios";

const apiBaseUrl = import.meta.env.VITE_API_BASE_URL ?? "/api";

const api = axios.create({
    baseURL: apiBaseUrl,
    withCredentials: true,
});

type RetryableRequestConfig = InternalAxiosRequestConfig & {
    _retry?: boolean;
};

let inFlightRefreshPromise: Promise<void> | null = null;

const refreshAccessToken = async () => {
    if (!inFlightRefreshPromise) {
        inFlightRefreshPromise = api
            .post("/auth/refresh")
            .then(() => undefined)
            .finally(() => {
                inFlightRefreshPromise = null;
            });
    }

    return inFlightRefreshPromise;
};

api.interceptors.response.use(
    (response) => response,
    async (error) => {
        const requestConfig = error.config as RetryableRequestConfig | undefined;
        const statusCode = error.response?.status as number | undefined;
        const requestUrl = requestConfig?.url ?? "";

        if (
            !requestConfig ||
            statusCode !== 401 ||
            requestConfig._retry ||
            requestUrl.includes("/auth/log-in") ||
            requestUrl.includes("/auth/refresh")
        ) {
            throw error;
        }

        requestConfig._retry = true;
        await refreshAccessToken();
        return api(requestConfig);
    },
);

export default api;
