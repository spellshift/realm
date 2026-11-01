import {
    HOST_FILTER_FIELDS,
    HostFilterFieldDef,
} from "../../../components/host-filter-bar/constants";
import {
    HostFilterField,
    HostFilterPredicate,
    HostFilterRow,
} from "../../../utils/interfacesUI";

// Rule field subset (TAGRULE_DECISIONS.md decision 3): host-level fields only.
// Excluded: status (meaningless at registration), beacon-level fields
// (transport, principal, beacon), and tag conditions (service, group).
export const TAG_RULE_FIELDS: HostFilterFieldDef[] = HOST_FILTER_FIELDS.filter(
    (def) =>
        def.value === "platform" ||
        def.value === "primaryIP" ||
        def.value === "externalIP" ||
        def.value === "host"
);

export const TAG_RULE_FIELD_VALUES: HostFilterField[] = TAG_RULE_FIELDS.map(
    (def) => def.value
);

const SINGLE_VALUE_PREDICATES: HostFilterPredicate[] = [
    "is",
    "is-not",
    "contains",
    "does-not-contain",
];

// Predicates allowed for a rule field. Platform is enum-like: no substring
// predicates, mirroring getPredicatesForField in host-filter-bar/constants.
export function getTagRulePredicates(
    field: HostFilterField
): HostFilterPredicate[] {
    if (field === "platform") {
        return ["is", "is-not", "is-in", "is-not-in"];
    }
    return [
        "is",
        "is-not",
        "is-in",
        "is-not-in",
        "contains",
        "does-not-contain",
    ];
}

export interface TagRuleValidationError {
    rowId: string;
    message: string;
}

// Validates rule rows before sending them to the backend. Mirrors the Go
// validator in internal/ent/schema/tagrule.go and internal/tagrules.
export function validateTagRuleRows(
    rows: HostFilterRow[]
): TagRuleValidationError[] {
    const errors: TagRuleValidationError[] = [];
    if (rows.length < 1) {
        return [{ rowId: "", message: "Add at least one condition row." }];
    }
    for (const row of rows) {
        if (!TAG_RULE_FIELD_VALUES.includes(row.field)) {
            errors.push({
                rowId: row.id,
                message: `Unsupported field "${row.field}".`,
            });
            continue;
        }
        if (!getTagRulePredicates(row.field).includes(row.predicate)) {
            errors.push({
                rowId: row.id,
                message: `Predicate "${row.predicate}" is not allowed for field "${row.field}".`,
            });
            continue;
        }
        if (row.values.length < 1 || row.values.some((v) => v.trim() === "")) {
            errors.push({
                rowId: row.id,
                message: "Each condition needs at least one non-empty value.",
            });
            continue;
        }
        if (
            SINGLE_VALUE_PREDICATES.includes(row.predicate) &&
            row.values.length !== 1
        ) {
            errors.push({
                rowId: row.id,
                message: `Predicate "${row.predicate}" takes exactly one value.`,
            });
        }
    }
    return errors;
}

export interface TagRuleConditionInput {
    field: string;
    predicate: string;
    values: string[];
}

// Converts editor rows to the createTagRule/updateTagRule GraphQL input.
export function toTagRuleConditionInputs(
    rows: HostFilterRow[]
): TagRuleConditionInput[] {
    return rows.map((row) => ({
        field: row.field,
        predicate: row.predicate,
        values: [...row.values],
    }));
}
