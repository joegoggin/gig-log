import type { Meta, StoryObj } from "@storybook/react-vite";
import RootLayout from "@/layouts/RootLayout/RootLayout";

const meta: Meta<typeof RootLayout> = {
    title: "Layouts/RootLayout",
    component: RootLayout,
    tags: ["autodocs"],
    parameters: {
        layout: "fullscreen",
    },
};

export default meta;
type Story = StoryObj<typeof RootLayout>;

export const Default: Story = {
    args: {
        children: (
            <div style={{ padding: "2rem" }}>
                <h1>Page Content</h1>
                <p>This is sample content inside the RootLayout.</p>
            </div>
        ),
    },
};

export const WithClassName: Story = {
    args: {
        className: "custom-page",
        children: (
            <div style={{ padding: "2rem" }}>
                <h1>Custom Page</h1>
                <p>This layout has an additional custom class applied.</p>
            </div>
        ),
    },
};
