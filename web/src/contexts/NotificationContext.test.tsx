/**
 * Unit tests for notification context ID generation behavior.
 *
 * Covered scenarios:
 * - Uses `crypto.randomUUID` when available.
 * - Falls back to a generated ID when `randomUUID` throws.
 *
 * These tests prevent regressions where notifications fail to appear because
 * ID generation throws before state updates complete.
 */
import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { NotificationType } from "@/components/core/Notification/Notification";
import { NotificationProvider, useNotification } from "@/contexts/NotificationContext";

type WrapperProps = {
    children: ReactNode;
};

const wrapper = ({ children }: WrapperProps) => {
    return <NotificationProvider>{children}</NotificationProvider>;
};

describe("NotificationContext", () => {
    afterEach(() => {
        vi.restoreAllMocks();
    });

    it("uses crypto.randomUUID when it succeeds", () => {
        const randomUuidSpy = vi
            .spyOn(globalThis.crypto, "randomUUID")
            .mockReturnValue("11111111-1111-1111-1111-111111111111");
        const { result } = renderHook(() => useNotification(), { wrapper });

        act(() => {
            result.current.addNotification({
                type: NotificationType.SUCCESS,
                title: "Job Created",
                message: "Your job has been created successfully.",
            });
        });

        expect(randomUuidSpy).toHaveBeenCalledTimes(1);
        expect(result.current.notifications).toHaveLength(1);
        expect(result.current.notifications[0]?.id).toBe(
            "11111111-1111-1111-1111-111111111111",
        );
    });

    it("falls back to generated IDs when randomUUID throws", () => {
        vi.spyOn(globalThis.crypto, "randomUUID").mockImplementation(() => {
            throw new Error("randomUUID unavailable");
        });
        const { result } = renderHook(() => useNotification(), { wrapper });

        act(() => {
            result.current.addNotification({
                type: NotificationType.SUCCESS,
                title: "Job Created",
                message: "Your job has been created successfully.",
            });
        });

        expect(result.current.notifications).toHaveLength(1);
        expect(result.current.notifications[0]?.id).toMatch(/^notification-\d+-[a-z0-9]+$/);
    });

    it("falls back to generated IDs when crypto is unavailable", () => {
        const originalCryptoDescriptor = Object.getOwnPropertyDescriptor(globalThis, "crypto");

        Object.defineProperty(globalThis, "crypto", {
            value: undefined,
            configurable: true,
        });

        try {
            const { result } = renderHook(() => useNotification(), { wrapper });

            act(() => {
                result.current.addNotification({
                    type: NotificationType.SUCCESS,
                    title: "Job Created",
                    message: "Your job has been created successfully.",
                });
            });

            expect(result.current.notifications).toHaveLength(1);
            expect(result.current.notifications[0]?.id).toMatch(/^notification-\d+-[a-z0-9]+$/);
        } finally {
            if (originalCryptoDescriptor) {
                Object.defineProperty(globalThis, "crypto", originalCryptoDescriptor);
            }
        }
    });
});
