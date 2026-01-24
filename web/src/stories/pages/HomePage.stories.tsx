import type { Meta, StoryObj } from "@storybook/react-vite";
import HomePage from "@/pages/HomePage/HomePage";

const meta: Meta<typeof HomePage> = {
    title: "Pages/HomePage",
    component: HomePage,
    tags: ["autodocs"],
    parameters: {
        layout: "fullscreen",
    },
};

export default meta;
type Story = StoryObj<typeof HomePage>;

export const LoggedOut: Story = {
    args: {
        isLoggedIn: false,
    },
};

export const LoggedIn: Story = {
    args: {
        isLoggedIn: true,
    },
};
