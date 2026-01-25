import type { Meta, StoryObj } from "@storybook/react-vite";
import Spinner from "@/components/core/Spinner/Spinner";

const meta: Meta<typeof Spinner> = {
    title: "Core/Spinner",
    component: Spinner,
    tags: ["autodocs"],
    argTypes: {
        size: {
            control: { type: "number" },
        },
    },
};

export default meta;
type Story = StoryObj<typeof Spinner>;

export const Default: Story = {
    args: {
        label: "Loading",
    },
};

export const Large: Story = {
    args: {
        label: "Loading",
        size: 64,
    },
};

export const CustomLabel: Story = {
    args: {
        label: "Fetching gigs",
    },
};
