import { FieldInputParams } from "../../../utils/interfacesUI";
import { safelyJsonParse } from "../../../utils/utils";

export type StoredParams = {
    values: Record<string, string>;
    invalid: boolean;
};

export const parseStoredParams = (stored: string | null | undefined): StoredParams => {
    if (!stored) return { values: {}, invalid: false };
    try {
        const parsed = JSON.parse(stored);
        if (!parsed || Array.isArray(parsed) || typeof parsed !== "object") return { values: {}, invalid: true };
        const values: Record<string, string> = {};
        Object.entries(parsed).forEach(([key, value]) => {
            values[key] = typeof value === "string" ? value : JSON.stringify(value);
        });
        return { values, invalid: false };
    } catch {
        return { values: {}, invalid: true };
    }
};

export const buildParamFields = (paramDefs: string | null | undefined, stored?: string | null): { fields: FieldInputParams[]; notice: boolean } => {
    const { params } = safelyJsonParse(paramDefs || "");
    const definitions: FieldInputParams[] = Array.isArray(params) ? params : [];
    const { values, invalid } = parseStoredParams(stored);
    const names = new Set(definitions.map((definition) => definition.name));
    const dropped = Object.keys(values).some((key) => !names.has(key));
    const fields = definitions.map((definition) => ({
        ...definition,
        value: stored !== undefined && stored !== null && definition.name in values ? values[definition.name] : definition.value ?? "",
    }));
    return { fields, notice: invalid || dropped };
};

export const serializeParams = (fields: FieldInputParams[]): string =>
    JSON.stringify(Object.fromEntries(fields.map((field) => [field.name, field.value ?? ""])));
