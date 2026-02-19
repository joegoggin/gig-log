import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { WorkSessionProvider, useWorkSession } from "./WorkSessionContext";
import type { ReactNode } from "react";
import type { WorkSession } from "@/types/models/WorkSession";
import api from "@/lib/axios";

vi.mock("@/lib/axios");
vi.mock("./AuthContext", () => ({
    useAuth: () => ({ isLoggedIn: true }),
}));

const wrapper = ({ children }: { children: ReactNode }) => (
    <WorkSessionProvider>{children}</WorkSessionProvider>
);

const mockSession: WorkSession = {
    id: "123",
    user_id: "user-1",
    job_id: "job-1",
    start_time: "2023-01-01T10:00:00Z",
    end_time: null,
    is_running: true,
    accumulated_paused_duration: 0,
    paused_at: null,
    time_reported: false,
    created_at: "2023-01-01T10:00:00Z",
    updated_at: "2023-01-01T10:00:00Z",
};

describe("WorkSessionContext", () => {
    beforeEach(() => {
        vi.useFakeTimers();
        vi.setSystemTime(new Date("2023-01-01T10:00:00Z"));
        vi.clearAllMocks();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it("fetches active session on mount", async () => {
        vi.mocked(api.get).mockResolvedValueOnce({
            data: { work_session: mockSession },
        } as any);

        const { result } = renderHook(() => useWorkSession(), { wrapper });

        expect(result.current.isLoading).toBe(true);

        // Let the useEffect promise resolve
        await act(async () => {
            await vi.runAllTimersAsync();
        });

        expect(result.current.isLoading).toBe(false);
        expect(result.current.activeSession).toEqual(mockSession);
        expect(api.get).toHaveBeenCalledWith("/work-sessions/active");
    });

    it("handles 404 when no active session exists", async () => {
        vi.mocked(api.get).mockRejectedValueOnce({
            response: { status: 404 },
        });

        const { result } = renderHook(() => useWorkSession(), { wrapper });

        await act(async () => {
            await vi.runAllTimersAsync();
        });

        expect(result.current.isLoading).toBe(false);
        expect(result.current.activeSession).toBeNull();
    });

    it("updates timer value while session is running", async () => {
        vi.mocked(api.get).mockResolvedValueOnce({
            data: { work_session: mockSession },
        } as any);

        const { result } = renderHook(() => useWorkSession(), { wrapper });

        await act(async () => {
            await vi.runAllTimersAsync(); // resolves fetch
        });

        expect(result.current.timerValue).toBe(0);

        act(() => {
            vi.advanceTimersByTime(5000);
        });

        expect(result.current.timerValue).toBe(5);
    });

    it("startSession sets active session", async () => {
        vi.mocked(api.get).mockRejectedValueOnce({ response: { status: 404 } });
        const { result } = renderHook(() => useWorkSession(), { wrapper });

        await act(async () => {
            await vi.runAllTimersAsync();
        });

        expect(result.current.isLoading).toBe(false);

        vi.mocked(api.post).mockResolvedValueOnce({
            data: { work_session: mockSession },
        } as any);

        await act(async () => {
            await result.current.startSession("job-1");
        });

        expect(api.post).toHaveBeenCalledWith("/work-sessions/start", {
            job_id: "job-1",
        });
        expect(result.current.activeSession).toEqual(mockSession);
    });

    it("pauseSession updates session", async () => {
        vi.mocked(api.get).mockResolvedValueOnce({
            data: { work_session: mockSession },
        } as any);

        const { result } = renderHook(() => useWorkSession(), { wrapper });

        await act(async () => {
            await vi.runAllTimersAsync();
        });

        const pausedSession = {
            ...mockSession,
            is_running: false,
            paused_at: "2023-01-01T10:05:00Z",
        };
        vi.mocked(api.post).mockResolvedValueOnce({
            data: { work_session: pausedSession },
        } as any);

        await act(async () => {
            await result.current.pauseSession("123");
        });

        expect(api.post).toHaveBeenCalledWith("/work-sessions/123/pause");
        expect(result.current.activeSession).toEqual(pausedSession);
    });

    it("resumeSession updates session", async () => {
        const pausedSession = {
            ...mockSession,
            is_running: false,
            paused_at: "2023-01-01T10:05:00Z",
        };
        vi.mocked(api.get).mockResolvedValueOnce({
            data: { work_session: pausedSession },
        } as any);

        const { result } = renderHook(() => useWorkSession(), { wrapper });

        await act(async () => {
            await vi.runAllTimersAsync();
        });

        vi.mocked(api.post).mockResolvedValueOnce({
            data: { work_session: mockSession },
        } as any);

        await act(async () => {
            await result.current.resumeSession("123");
        });

        expect(api.post).toHaveBeenCalledWith("/work-sessions/123/resume");
        expect(result.current.activeSession).toEqual(mockSession);
    });

    it("completeSession clears session", async () => {
        vi.mocked(api.get).mockResolvedValueOnce({
            data: { work_session: mockSession },
        } as any);

        const { result } = renderHook(() => useWorkSession(), { wrapper });

        await act(async () => {
            await vi.runAllTimersAsync();
        });

        vi.mocked(api.post).mockResolvedValueOnce({
            data: {
                work_session: {
                    ...mockSession,
                    is_running: false,
                    end_time: "2023-01-01T10:05:00Z",
                },
            },
        } as any);

        await act(async () => {
            await result.current.completeSession("123");
        });

        expect(api.post).toHaveBeenCalledWith("/work-sessions/123/complete");
        expect(result.current.activeSession).toBeNull();
    });
});
