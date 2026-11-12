import { describe, expect, it } from "vitest";
import { uniqueJsonError } from "../validation";

describe("uniqueJsonError", () => {
    it("accepts blank values so the agent falls back to its defaults", () => {
        expect(uniqueJsonError("")).toBeNull();
        expect(uniqueJsonError("   ")).toBeNull();
    });

    it("accepts valid JSON selector arrays", () => {
        expect(uniqueJsonError('[{"type":"env"},{"type":"file"}]')).toBeNull();
    });

    it("rejects values that are not JSON, such as a bare identifier", () => {
        expect(uniqueJsonError("win-diag-02")).toMatch(/Not valid JSON/);
        expect(uniqueJsonError("[{type:env}]")).toMatch(/Not valid JSON/);
    });
});
