import { describe, expect, it } from "vitest";
import { capitalize } from "@/utils/capitalize";

describe("capitalize", () => {
    it("capitalizes the first character", () => {
        expect(capitalize("giglog")).toBe("Giglog");
    });

    it("returns an empty string when input is empty", () => {
        expect(capitalize("")).toBe("");
    });

    it("keeps the remainder of the string unchanged", () => {
        expect(capitalize("gIGlog")).toBe("GIGlog");
    });
});
