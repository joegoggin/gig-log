import { expect, userEvent, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import Link from "@/components/core/Link/Link";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof Link> = {
    title: "Core/Link",
    component: Link,
    tags: ["autodocs"],
    decorators: [withMemoryRouter],
    argTypes: {
        href: {
            control: { type: "text" },
        },
    },
};

export default meta;
type Story = StoryObj<typeof Link>;

export const Default: Story = {
    args: {
        href: "/dashboard",
        children: "Open dashboard",
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await userEvent.click(
            canvas.getByRole("button", { name: "Open dashboard" }),
        );
        await expect(canvas.getByText("Dashboard Route")).toBeVisible();
    },
};

export const SignUpLink: Story = {
    args: {
        href: "/auth/sign-up",
        children: "Create account",
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await userEvent.click(
            canvas.getByRole("button", { name: "Create account" }),
        );
        await expect(canvas.getByText("Sign Up Route")).toBeVisible();
    },
};
