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
            Object.defineProperty(values, key, { value: typeof value === "string" ? value : JSON.stringify(value), enumerable: true, writable: true, configurable: true });
        });
        return { values, invalid: false };
    } catch {
        return { values: {}, invalid: true };
    }
};

export type ParamNotice = "invalid" | "dropped" | null;

export const buildParamFields = (
    paramDefs: string | null | undefined,
    stored?: string | null,
): { fields: FieldInputParams[]; notice: ParamNotice; unavailable: boolean } => {
    const { params, error } = safelyJsonParse(paramDefs || "");
    const definitions: FieldInputParams[] = Array.isArray(params) ? params : [];
    // Definitions we could not read (as opposed to a tome with no params): never discard stored values because of them.
    const unavailable = error || (!!paramDefs && !Array.isArray(params));
    const { values, invalid } = parseStoredParams(stored);
    const names = new Set(definitions.map((definition) => definition.name));
    const dropped = !unavailable && Object.keys(values).some((key) => !names.has(key));
    const hasStored = stored !== undefined && stored !== null;
    const fields = definitions.map((definition) => ({
        ...definition,
        value: hasStored && Object.hasOwn(values, definition.name) ? values[definition.name] : definition.value ?? "",
    }));
    return { fields, notice: invalid ? "invalid" : dropped ? "dropped" : null, unavailable };
};

export const serializeParams = (fields: FieldInputParams[]): string =>
    JSON.stringify(Object.fromEntries(fields.map((field) => [field.name, field.value ?? ""])));
