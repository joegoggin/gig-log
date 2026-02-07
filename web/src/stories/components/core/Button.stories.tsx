import { expect, fn, userEvent, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import Button, { ButtonVariant } from "@/components/core/Button/Button";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof Button> = {
    title: "Core/Button",
    component: Button,
    tags: ["autodocs"],
    decorators: [withMemoryRouter],
    parameters: {
        a11y: {
            test: "error",
        },
    },
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
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(
            canvas.getByRole("button", { name: "Primary Button" }),
        ).toBeVisible();
    },
};

export const Secondary: Story = {
    args: {
        variant: ButtonVariant.SECONDARY,
        children: "Secondary Button",
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(
            canvas.getByRole("button", { name: "Secondary Button" }),
        ).toBeVisible();
    },
};

export const WithClickHandler: Story = {
    args: {
        variant: ButtonVariant.PRIMARY,
        children: "Click Me",
        onClick: fn(),
    },
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement);
        await userEvent.click(canvas.getByRole("button", { name: "Click Me" }));
        await expect(args.onClick).toHaveBeenCalledTimes(1);
    },
};

export const SubmitButton: Story = {
    args: {
        variant: ButtonVariant.PRIMARY,
        type: "submit",
        children: "Submit",
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByRole("button", { name: "Submit" })).toHaveAttribute(
            "type",
            "submit",
        );
    },
};

export const NavigatesToHref: Story = {
    args: {
        children: "Go to Dashboard",
        href: "/dashboard",
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await userEvent.click(
            canvas.getByRole("button", { name: "Go to Dashboard" }),
        );
        await expect(canvas.getByText("Dashboard Route")).toBeVisible();
    },
};
