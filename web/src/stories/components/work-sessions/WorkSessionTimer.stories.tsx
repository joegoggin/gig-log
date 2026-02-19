import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, fn, userEvent, within } from "storybook/test";
import WorkSessionTimer from "@/components/work-sessions/WorkSessionTimer";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";

const meta: Meta<typeof WorkSessionTimer> = {
    title: "Core/WorkSessionTimer",
    component: WorkSessionTimer,
    tags: ["autodocs"],
    decorators: [withMemoryRouter],
    argTypes: {
        status: {
            control: "select",
            options: ["idle", "running", "paused", "completed"],
        },
        startTime: { control: "text" },
        accumulatedSeconds: { control: "number" },
    },
};

export default meta;
type Story = StoryObj<typeof WorkSessionTimer>;

export const Idle: Story = {
    args: {
        status: "idle",
        accumulatedSeconds: 0,
        onStart: fn(),
    },
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement);
        const startButton = canvas.getByRole("button", { name: /start/i });
        await expect(startButton).toBeInTheDocument();
        await userEvent.click(startButton);
        await expect(args.onStart).toHaveBeenCalled();
    },
};

export const Running: Story = {
    args: {
        status: "running",
        // Simulate a timer that started 5 minutes ago (300 seconds)
        startTime: new Date(Date.now() - 300000).toISOString(),
        accumulatedSeconds: 0,
        onPause: fn(),
        onComplete: fn(),
    },
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement);
        
        // Assert timer shows time (e.g. 00:05:00)
        const display = canvas.getByText(/00:05:0/);
        await expect(display).toBeInTheDocument();

        // Pause
        const pauseButton = canvas.getByRole("button", { name: /pause/i });
        await expect(pauseButton).toBeInTheDocument();
        await userEvent.click(pauseButton);
        await expect(args.onPause).toHaveBeenCalled();

        // Complete
        const completeButton = canvas.getByRole("button", { name: /complete/i });
        await expect(completeButton).toBeInTheDocument();
        await userEvent.click(completeButton);
        await expect(args.onComplete).toHaveBeenCalled();
    },
};

export const Paused: Story = {
    args: {
        status: "paused",
        accumulatedSeconds: 3661, // 1h 1m 1s
        onResume: fn(),
        onComplete: fn(),
    },
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement);
        
        // Check display format
        await expect(canvas.getByText("01:01:01")).toBeInTheDocument();
        await expect(canvas.getByText(/paused/i)).toBeInTheDocument();

        // Resume
        const resumeButton = canvas.getByRole("button", { name: /resume/i });
        await expect(resumeButton).toBeInTheDocument();
        await userEvent.click(resumeButton);
        await expect(args.onResume).toHaveBeenCalled();
    },
};

export const Completed: Story = {
    args: {
        status: "completed",
        accumulatedSeconds: 7200, // 2h 0m 0s
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement);
        await expect(canvas.getByText("02:00:00")).toBeInTheDocument();
        await expect(canvas.getByText(/completed/i)).toBeInTheDocument();
        
        // No controls should be visible
        const buttons = canvas.queryAllByRole("button");
        await expect(buttons).toHaveLength(0);
    },
};
