import { describe, expect, it } from "vitest";
import { parseValidationErrors } from "@/lib/errors";

describe("parseValidationErrors", () => {
    it("maps validation errors by field", () => {
        const result = parseValidationErrors({
            errors: [
                { field: "email", message: "Email is invalid" },
                { field: "password", message: "Password is required" },
            ],
        });

        expect(result).toEqual({
            email: "Email is invalid",
            password: "Password is required",
        });
    });

    it("uses the last message for duplicate fields", () => {
        const result = parseValidationErrors({
            errors: [
                { field: "email", message: "Invalid email" },
                { field: "email", message: "Email already exists" },
            ],
        });

        expect(result).toEqual({
            email: "Email already exists",
        });
    });
});
