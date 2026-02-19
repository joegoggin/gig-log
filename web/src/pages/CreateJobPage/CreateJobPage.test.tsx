/**
 * Unit tests for Create Job page mutation side-effect behavior.
 *
 * Covered scenarios:
 * - Successful create-job submission posts normalized payload, dispatches a
 *   success notification, and clears form fields while preserving a
 *   preselected company.
 *
 * This test prevents regressions where the success flow appears to do nothing
 * because notifications fail or form reset logic stops running.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import CreateJobPage from "./CreateJobPage";
import type * as TanStackRouter from "@tanstack/react-router";
import { NotificationType } from "@/components/core/Notification/Notification";
import { NotificationContext } from "@/contexts/NotificationContext";
import {
    createMockApiResponse,
    mockApiGetHandler,
    mockApiPostHandler,
} from "@/test-utils/mockApiClient";

const navigateMock = vi.fn();

vi.mock("@tanstack/react-router", async () => {
    const actual = await vi.importActual<typeof TanStackRouter>("@tanstack/react-router");

    return {
        ...actual,
        useNavigate: () => navigateMock,
    };
});

let restoreGet: (() => void) | undefined;
let restorePost: (() => void) | undefined;

type NotificationCall = {
    type: NotificationType;
    title: string;
    message: string;
};

const companiesFixture = [
    {
        id: "11111111-1111-1111-1111-111111111111",
        user_id: "u1",
        name: "Acme Studio",
        requires_tax_withholdings: false,
        tax_withholding_rate: null,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
    },
    {
        id: "22222222-2222-2222-2222-222222222222",
        user_id: "u1",
        name: "Nova Labs",
        requires_tax_withholdings: false,
        tax_withholding_rate: null,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
    },
];

const renderPage = (addNotification: (notification: NotificationCall) => void) => {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: { retry: false },
            mutations: { retry: false },
        },
    });

    render(
        <QueryClientProvider client={queryClient}>
            <NotificationContext.Provider
                value={{
                    notifications: [],
                    addNotification,
                    removeNotification: () => {},
                }}
            >
                <CreateJobPage preselectedCompanyId="11111111-1111-1111-1111-111111111111" />
            </NotificationContext.Provider>
        </QueryClientProvider>,
    );
};

describe("CreateJobPage", () => {
    afterEach(() => {
        restoreGet?.();
        restoreGet = undefined;
        restorePost?.();
        restorePost = undefined;
        navigateMock.mockReset();
    });

    it("submits create payload, notifies success, and resets fields", async () => {
        const addNotification = vi.fn();
        const postCalls: Array<{ url: string; data: unknown }> = [];

        restoreGet = mockApiGetHandler((url) => {
            if (url === "/companies") {
                return Promise.resolve(createMockApiResponse({ companies: companiesFixture }));
            }

            return Promise.resolve(createMockApiResponse({}));
        });

        restorePost = mockApiPostHandler((url, data) => {
            postCalls.push({ url, data });
            return Promise.resolve(createMockApiResponse({ job: { id: "j1" } }, 201, "Created"));
        });

        renderPage(addNotification);

        await waitFor(() => {
            screen.getByRole("option", { name: "Acme Studio" });
        });

        expect((screen.getByLabelText("Company") as HTMLSelectElement).value).toBe(
            "11111111-1111-1111-1111-111111111111",
        );

        fireEvent.change(screen.getByPlaceholderText("Job Title"), {
            target: { value: "Website Retainer" },
        });
        fireEvent.change(screen.getByLabelText("Payment Type"), {
            target: { value: "payouts" },
        });
        fireEvent.change(screen.getByPlaceholderText("Number of Payouts"), {
            target: { value: "3" },
        });
        fireEvent.change(screen.getByPlaceholderText("Payout Amount"), {
            target: { value: "250.00" },
        });

        fireEvent.click(screen.getByRole("button", { name: "Create Job" }));

        await waitFor(() => {
            expect(postCalls).toHaveLength(1);
        });

        expect(postCalls[0]).toEqual({
            url: "/jobs",
            data: {
                company_id: "11111111-1111-1111-1111-111111111111",
                title: "Website Retainer",
                payment_type: "payouts",
                number_of_payouts: 3,
                payout_amount: "250.00",
                hourly_rate: null,
            },
        });
        expect(addNotification).toHaveBeenCalledWith({
            type: NotificationType.SUCCESS,
            title: "Job Created",
            message: "Your job has been created successfully.",
        });
        expect((screen.getByPlaceholderText("Job Title") as HTMLInputElement).value).toBe("");
        expect((screen.getByLabelText("Company") as HTMLSelectElement).value).toBe(
            "11111111-1111-1111-1111-111111111111",
        );
        expect((screen.getByLabelText("Payment Type") as HTMLSelectElement).value).toBe(
            "hourly",
        );
        expect(screen.queryByPlaceholderText("Number of Payouts")).toBeNull();
        expect(screen.queryByPlaceholderText("Payout Amount")).toBeNull();
        expect(screen.getByPlaceholderText("Hourly Rate")).toBeTruthy();
    });
});
