import { useState } from "react";
import { expect, fn, userEvent, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { SetData } from "@/types/SetData";
import CheckBox from "@/components/core/CheckBox/CheckBox";

type CheckBoxStoryProps = {
    className?: string;
    label: string;
    initialChecked: boolean;
    onToggle?: (checked: boolean) => void;
};

const CheckBoxStory = ({
    className,
    label,
    initialChecked,
    onToggle,
}: CheckBoxStoryProps) => {
    const [data, setDataState] = useState({ agreed: initialChecked });

    const setData: SetData<typeof data> = (key, value) => {
        setDataState((prev) => ({ ...prev, [key]: value }));
        onToggle?.(Boolean(value));
    };

    return (
        <CheckBox<typeof data>
            className={className}
            label={label}
            name="agreed"
            data={data}
            setData={setData}
        />
    );
};

const meta: Meta<typeof CheckBoxStory> = {
    title: "Core/CheckBox",
    component: CheckBoxStory,
    tags: ["autodocs"],
    argTypes: {
        initialChecked: {
            control: { type: "boolean" },
        },
    },
};

export default meta;
type Story = StoryObj<typeof CheckBoxStory>;

export const Default: Story = {
    args: {
        label: "I agree to the terms",
        initialChecked: false,
        onToggle: fn(),
    },
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement);
        const checkbox = canvas.getByRole("checkbox");
        await expect(checkbox).not.toBeChecked();
        await userEvent.click(checkbox);
        await expect(checkbox).toBeChecked();
        await expect(args.onToggle).toHaveBeenCalledWith(true);
    },
};

export const InitiallyChecked: Story = {
    args: {
        label: "Email me updates",
        initialChecked: true,
        onToggle: fn(),
    },
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement);
        const checkbox = canvas.getByRole("checkbox");
        await expect(checkbox).toBeChecked();
        await userEvent.click(checkbox);
        await expect(checkbox).not.toBeChecked();
        await expect(args.onToggle).toHaveBeenCalledWith(false);
    },
};
