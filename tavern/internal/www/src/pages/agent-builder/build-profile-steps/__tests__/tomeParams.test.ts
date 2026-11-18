import { describe, expect, it } from "vitest";
import { buildParamFields, serializeParams } from "../tomeParams";

const paramDefs = JSON.stringify([
    { name: "path", label: "Path", type: "string", placeholder: "/tmp" },
    { name: "mode", label: "Mode", type: "string", placeholder: "", value: "default" },
    { name: "constructor", label: "Ctor", type: "string", placeholder: "" },
]);

describe("tome params round trip", () => {
    it("prefills, drops unknown keys, flags bad input and re-serializes", () => {
        // Prefill + drop unknown + non-string coercion, and prototype-named params don't read Object.prototype.
        const stored = JSON.stringify({ path: "/etc", removed: "x", mode: 5 });
        const { fields, notice, unavailable } = buildParamFields(paramDefs, stored);
        expect(notice).toBe("dropped");
        expect(unavailable).toBe(false);
        expect(fields.map((field) => [field.name, field.value])).toEqual([["path", "/etc"], ["mode", "5"], ["constructor", ""]]);

        const edited = fields.map((field) => (field.name === "mode" ? { ...field, value: "fast" } : field));
        expect(serializeParams(edited)).toBe(JSON.stringify({ path: "/etc", mode: "fast", constructor: "" }));

        // Invalid / non-object stored JSON: defaults + "invalid" notice (not "dropped").
        expect(buildParamFields(paramDefs, "not json").notice).toBe("invalid");
        expect(buildParamFields(paramDefs, "[1]").notice).toBe("invalid");
        expect(buildParamFields(paramDefs, "null").notice).toBe("invalid");
        expect(buildParamFields(paramDefs, "").notice).toBeNull();

        // Unreadable definitions never count stored keys as dropped.
        const bad = buildParamFields("{oops", JSON.stringify({ path: "/etc" }));
        expect(bad.unavailable).toBe(true);
        expect(bad.notice).toBeNull();

        // Tome with no params.
        const empty = buildParamFields("", "{}");
        expect(empty.notice).toBeNull();
        expect(empty.unavailable).toBe(false);
        expect(serializeParams(empty.fields)).toBe("{}");
    });
});
