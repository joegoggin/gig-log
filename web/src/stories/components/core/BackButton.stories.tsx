import { expect, userEvent, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import BackButton from "@/components/core/BackButton/BackButton";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof BackButton> = {
    title: "Core/BackButton",
    component: BackButton,
    tags: ["autodocs"],
    decorators: [withMemoryRouter],
    parameters: {
        a11y: {
            test: "error",
        },
    },
};

export default meta;
type Story = StoryObj<typeof BackButton>;

export const Default: Story = {
    args: {
        href: "/companies",
        children: "Back to Companies",
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(
            canvas.getByRole("button", { name: "Back to Companies" }),
        ).toBeVisible();
    },
};

export const NavigatesToHref: Story = {
    args: {
        href: "/dashboard",
        children: "Back to Dashboard",
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await userEvent.click(
            canvas.getByRole("button", { name: "Back to Dashboard" }),
        );
        await expect(canvas.getByText("Dashboard Route")).toBeVisible();
    },
};
