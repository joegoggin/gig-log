import { URL, fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import { devtools } from "@tanstack/devtools-vite";
import viteReact from "@vitejs/plugin-react";
import typedCssModules from "unplugin-typed-css-modules/vite";

import { tanstackRouter } from "@tanstack/router-plugin/vite";

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [
        devtools(),
        tanstackRouter({
            target: "react",
            autoCodeSplitting: true,
        }),
        viteReact(),
        typedCssModules({
            scss: true,
        }),
    ],
    resolve: {
        alias: {
            "@": fileURLToPath(new URL("./src", import.meta.url)),
            "@sass": fileURLToPath(new URL("./src/sass", import.meta.url)),
        },
    },
});
