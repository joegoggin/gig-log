import type { Meta, StoryObj } from "@storybook/react-vite";
import Button, { ButtonVariant } from "@/components/core/Button/Button";

const meta: Meta<typeof Button> = {
    title: "Core/Button",
    component: Button,
    tags: ["autodocs"],
    argTypes: {
        variant: {
            control: { type: "select" },
            options: [ButtonVariant.PRIMARY, ButtonVariant.SECONDARY],
        },
        type: {
            control: { type: "select" },
            options: ["button", "submit", "reset"],
        },
    },
};

export default meta;
type Story = StoryObj<typeof Button>;

export const Primary: Story = {
    args: {
        variant: ButtonVariant.PRIMARY,
        children: "Primary Button",
    },
};

export const Secondary: Story = {
    args: {
        variant: ButtonVariant.SECONDARY,
        children: "Secondary Button",
    },
};

export const WithClickHandler: Story = {
    args: {
        variant: ButtonVariant.PRIMARY,
        children: "Click Me",
        onClick: () => alert("Button clicked!"),
    },
};

export const SubmitButton: Story = {
    args: {
        variant: ButtonVariant.PRIMARY,
        type: "submit",
        children: "Submit",
    },
};
