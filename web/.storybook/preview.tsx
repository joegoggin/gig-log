import { withThemeByDataAttribute } from "@storybook/addon-themes";
import "../src/sass/index.scss";
import theme from "./theme";
import type { Preview } from "@storybook/react-vite";

const preview: Preview = {
    initialGlobals: {
        theme: "dark",
    },

    decorators: [
        withThemeByDataAttribute({
            themes: {
                light: "light",
                dark: "dark",
            },
            defaultTheme: "dark",
            attributeName: "data-theme",
        }),
        (Story) => (
            <div style={{ background: "var(--bg-color)", padding: "2rem" }}>
                <Story />
            </div>
        ),
    ],

    parameters: {
        docs: {
            theme,
        },

        backgrounds: { disabled: true },

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
