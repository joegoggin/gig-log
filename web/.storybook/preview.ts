import type { Preview } from "@storybook/react-vite";
import "../src/sass/index.scss";
import theme from "./theme";

const preview: Preview = {
    initialGlobals: {
        backgrounds: { value: "dark" },
    },

    parameters: {
        docs: {
            theme,
        },

        backgrounds: {
            options: {
                dark: { name: "dark", value: "#1a1a1a" },
                light: { name: "light", value: "#ffffff" },
            },
        },

        controls: {
            matchers: {
                color: /(background|color)$/i,
                date: /Date$/i,
            },
        },

        a11y: {
            // 'todo' - show a11y violations in the test UI only
            // 'error' - fail CI on a11y violations
            // 'off' - skip a11y checks entirely
            test: "todo",
        },
    },
};

export default preview;
