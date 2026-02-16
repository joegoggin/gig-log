/**
 * Storybook interaction tests for Create Company page behavior.
 *
 * Covered scenarios:
 * - Successful submission sends expected payload and navigates to companies.
 * - Validation errors from the API are rendered at field level.
 */
import { expect, fn, userEvent, waitFor, within } from "storybook/test";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { StoryTestParameters } from "@/stories/testing/storyTestContext";
import { NotificationType } from "@/components/core/Notification/Notification";
import CreateCompanyPage from "@/pages/CreateCompanyPage/CreateCompanyPage";
import withAppProviders from "@/stories/decorators/withAppProviders";
import withMemoryRouter from "@/stories/decorators/withMemoryRouter";
import {
    createMockApiResponse,
    createValidationAxiosError,
    mockApiPostHandler,
} from "@/test-utils/mockApiClient";

const addNotificationSpy = fn();

type PostCall = {
    url: string;
    data: unknown;
};

const meta: Meta<typeof CreateCompanyPage> = {
    title: "Pages/CreateCompanyPage",
    component: CreateCompanyPage,
    tags: ["autodocs"],
    decorators: [withMemoryRouter, withAppProviders],
    parameters: {
        layout: "fullscreen",
        storyTest: {
            router: {
                storyPath: "/companies/create",
                initialEntries: ["/companies/create"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
        },
    },
};

export default meta;
type Story = StoryObj<typeof CreateCompanyPage>;

export const SubmitsPayloadAndNavigates: Story = {
    parameters: {
        storyTest: {
            router: {
                storyPath: "/companies/create",
                initialEntries: ["/companies/create"],
            },
            auth: {
                isLoggedIn: true,
                isLoading: false,
            },
            spies: {
                addNotification: addNotificationSpy,
            },
        },
    } satisfies StoryTestParameters,
    play: async ({ canvasElement }) => {
        const postCalls: Array<PostCall> = [];
        const restorePost = mockApiPostHandler(async (url, data) => {
            postCalls.push({ url, data });
            return createMockApiResponse({ message: "Created" }, 201, "Created");
        });

        addNotificationSpy.mockClear();

        try {
            const canvas = within(canvasElement);
            await userEvent.type(canvas.getByPlaceholderText("Company Name"), "Acme Studio");
            await userEvent.click(canvas.getByRole("checkbox"));
            await userEvent.type(
                canvas.getByPlaceholderText("Tax Withholding Rate"),
                "30.00",
            );
            await userEvent.click(canvas.getByRole("button", { name: "Create Company" }));

            await waitFor(() => {
                expect(postCalls).toHaveLength(1);
            });
            await expect(postCalls[0]).toEqual({
                url: "/companies",
                data: {
                    name: "Acme Studio",
                    requires_tax_withholdings: true,
                    tax_withholding_rate: "30.00",
                },
            });
            await expect(addNotificationSpy).toHaveBeenCalledWith({
                type: NotificationType.SUCCESS,
                title: "Company Created",
                message: "Your company has been created successfully.",
            });
            await expect(canvas.getByText("Companies Route")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};

export const ShowsValidationErrorsFromApi: Story = {
    play: async ({ canvasElement }) => {
        const restorePost = mockApiPostHandler(async () => {
            throw createValidationAxiosError([
                { field: "name", message: "Company name is required" },
            ]);
        });

        try {
            const canvas = within(canvasElement);
            await userEvent.click(canvas.getByRole("button", { name: "Create Company" }));
            await expect(canvas.getByText("Company name is required")).toBeVisible();
        } finally {
            restorePost();
        }
    },
};
