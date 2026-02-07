import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import useForm from "@/hooks/useForm";

type DemoForm = {
    email: string;
    remember: boolean;
};

describe("useForm", () => {
    it("updates field data and clears existing errors", () => {
        const { result } = renderHook(() =>
            useForm<DemoForm>({ email: "", remember: false }),
        );

        act(() => {
            result.current.setErrors({ email: "Email is required" });
        });

        expect(result.current.errors).toEqual({ email: "Email is required" });

        act(() => {
            result.current.setData("email", "demo@example.com");
        });

        expect(result.current.data).toEqual({
            email: "demo@example.com",
            remember: false,
        });
        expect(result.current.errors).toEqual({});
    });

    it("updates boolean values", () => {
        const { result } = renderHook(() =>
            useForm<DemoForm>({ email: "", remember: false }),
        );

        act(() => {
            result.current.setData("remember", true);
        });

        expect(result.current.data.remember).toBe(true);
    });
});
