import { describe, it, expect } from "vitest";
import { HostFilterRow } from "../../../../utils/interfacesUI";
import {
    TAG_RULE_FIELDS,
    TAG_RULE_FIELD_VALUES,
    getTagRulePredicates,
    toTagRuleConditionInputs,
    validateTagRuleRows,
} from "../constants";
import { parseTagRuleConditions } from "../queries";

const row = (overrides: Partial<HostFilterRow> = {}): HostFilterRow => ({
    id: "row-1",
    field: "platform",
    predicate: "is",
    values: ["PLATFORM_LINUX"],
    ...overrides,
});

describe("TAG_RULE_FIELDS subset", () => {
    it("contains only host-level fields", () => {
        expect(TAG_RULE_FIELD_VALUES).toEqual([
            "platform",
            "primaryIP",
            "externalIP",
            "host",
        ]);
    });

    it("excludes status, beacon-level, and tag fields", () => {
        const values = TAG_RULE_FIELDS.map((def) => def.value);
        for (const excluded of [
            "status",
            "transport",
            "principal",
            "beacon",
            "service",
            "group",
        ]) {
            expect(values).not.toContain(excluded);
        }
    });
});

describe("getTagRulePredicates", () => {
    it("excludes substring predicates for the platform enum field", () => {
        expect(getTagRulePredicates("platform")).toEqual([
            "is",
            "is-not",
            "is-in",
            "is-not-in",
        ]);
    });

    it("allows all six predicates on free-text fields", () => {
        for (const field of ["primaryIP", "externalIP", "host"] as const) {
            expect(getTagRulePredicates(field)).toEqual([
                "is",
                "is-not",
                "is-in",
                "is-not-in",
                "contains",
                "does-not-contain",
            ]);
        }
    });
});

describe("validateTagRuleRows", () => {
    it("requires at least one row", () => {
        expect(validateTagRuleRows([])).toHaveLength(1);
    });

    it("accepts a valid platform + host combination", () => {
        expect(
            validateTagRuleRows([
                row(),
                row({
                    id: "row-2",
                    field: "host",
                    predicate: "contains",
                    values: ["web"],
                }),
            ])
        ).toEqual([]);
    });

    it("rejects excluded fields", () => {
        const errors = validateTagRuleRows([
            row({ field: "status" as HostFilterRow["field"] }),
        ]);
        expect(errors).toHaveLength(1);
    });

    it("rejects contains on platform", () => {
        const errors = validateTagRuleRows([
            row({ predicate: "contains", values: ["LINUX"] }),
        ]);
        expect(errors).toHaveLength(1);
    });

    it("rejects empty and multi values for single-value predicates", () => {
        expect(
            validateTagRuleRows([row({ values: [] })])
        ).toHaveLength(1);
        expect(
            validateTagRuleRows([
                row({ predicate: "is", values: ["a", "b"] }),
            ])
        ).toHaveLength(1);
    });

    it("accepts multi values for is-in", () => {
        expect(
            validateTagRuleRows([
                row({
                    field: "primaryIP",
                    predicate: "is-in",
                    values: ["10.0.0.1", "10.0.0.2"],
                }),
            ])
        ).toEqual([]);
    });
});

describe("toTagRuleConditionInputs", () => {
    it("maps rows to GraphQL inputs", () => {
        expect(
            toTagRuleConditionInputs([
                row(),
                row({
                    id: "row-2",
                    field: "host",
                    predicate: "contains",
                    values: ["web"],
                }),
            ])
        ).toEqual([
            {
                field: "platform",
                predicate: "is",
                values: ["PLATFORM_LINUX"],
            },
            { field: "host", predicate: "contains", values: ["web"] },
        ]);
    });
});

describe("parseTagRuleConditions", () => {
    it("round-trips stored conditions to editor rows", () => {
        const stored =
            '[{"field":"platform","predicate":"is","values":["PLATFORM_LINUX"]}]';
        const rows = parseTagRuleConditions(stored);
        expect(rows).toHaveLength(1);
        expect(rows[0]).toMatchObject({
            field: "platform",
            predicate: "is",
            values: ["PLATFORM_LINUX"],
        });
    });

    it("returns empty rows for invalid JSON", () => {
        expect(parseTagRuleConditions("not-json")).toEqual([]);
    });
});
