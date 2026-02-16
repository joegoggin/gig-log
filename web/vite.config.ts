/// <reference types="vitest/config" />
import path from "node:path";
import { URL, fileURLToPath } from "node:url";
import { storybookTest } from "@storybook/addon-vitest/vitest-plugin";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// https://vitejs.dev/config/
const dirname =
    typeof __dirname !== "undefined"
        ? __dirname
        : path.dirname(fileURLToPath(import.meta.url));

// More info at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon
export default defineConfig({
    plugins: [
        devtools(),
        tanstackRouter({
            target: "react",
            autoCodeSplitting: true,
            routeFileIgnorePattern: "(__stories__|.*\\.stories\\.tsx?)$",
        }),
        viteReact(),
    ],
    resolve: {
        alias: {
            "@": fileURLToPath(new URL("./src", import.meta.url)),
            "@sass": fileURLToPath(new URL("./src/sass", import.meta.url)),
        },
    },
    server: {
        proxy: {
            "/api": {
                target: "http://127.0.0.1:8000",
                changeOrigin: true,
                cookieDomainRewrite: "",
                cookiePathRewrite: {
                    "/auth": "/api/auth",
                },
                rewrite: (path) => path.replace(/^\/api/, ""),
            },
        },
    },
    test: {
        projects: [
            {
                extends: true,
                plugins: [
                    // The plugin will run tests for the stories defined in your Storybook config
                    // See options at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon#storybooktest
                    storybookTest({
                        configDir: path.join(dirname, ".storybook"),
                    }),
                ],
                test: {
                    name: "storybook",
                    retry: process.env.CI ? 2 : 0,
                    testTimeout: 15000,
                    browser: {
                        enabled: true,
                        headless: true,
                        provider: "playwright",
                        instances: [
                            {
                                browser: "chromium",
                            },
                        ],
                    },
                    setupFiles: [".storybook/vitest.setup.ts"],
                },
            },
            {
                extends: true,
                test: {
                    name: "unit",
                    environment: "jsdom",
                    include: ["src/**/*.{test,spec}.{ts,tsx}"],
                    exclude: ["src/**/*.stories.{ts,tsx}"],
                    setupFiles: ["./vitest.setup.ts"],
                    testTimeout: 10000,
                },
            },
        ],
    },
});
