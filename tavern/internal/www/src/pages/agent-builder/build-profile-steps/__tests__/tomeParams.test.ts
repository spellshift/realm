import { describe, expect, it } from "vitest";
import { buildParamFields, serializeParams } from "../tomeParams";

const paramDefs = JSON.stringify([
    { name: "path", label: "Path", type: "string", placeholder: "/tmp" },
    { name: "mode", label: "Mode", type: "string", placeholder: "", value: "default" },
]);

describe("tome params round trip", () => {
    it("prefills from stored JSON, drops unknown keys and re-serializes", () => {
        const stored = JSON.stringify({ path: "/etc", removed: "x" });
        const { fields, notice } = buildParamFields(paramDefs, stored);

        expect(notice).toBe(true);
        expect(fields.map((field) => [field.name, field.value])).toEqual([["path", "/etc"], ["mode", "default"]]);

        const edited = fields.map((field) => (field.name === "mode" ? { ...field, value: "fast" } : field));
        expect(serializeParams(edited)).toBe(JSON.stringify({ path: "/etc", mode: "fast" }));
    });

    it("flags invalid stored JSON and serializes empty defs as {}", () => {
        expect(buildParamFields(paramDefs, "not json").notice).toBe(true);
        const { fields, notice } = buildParamFields("", "{}");
        expect(notice).toBe(false);
        expect(serializeParams(fields)).toBe("{}");
    });
});
