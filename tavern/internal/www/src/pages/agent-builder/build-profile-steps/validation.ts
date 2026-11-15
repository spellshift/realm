/** Index of the "Unique (JSON, optional)" entry in `advancedScriptFields`. */
export const UNIQUE_FIELD_INDEX = 5;

/**
 * Validates the optional profile `unique` value, which is passed to the agent
 * as IMIX_UNIQUE and must be JSON. Blank is valid (the agent uses defaults).
 * Returns an error message, or null when the value is acceptable.
 */
export const uniqueJsonError = (value: string): string | null => {
    if (!value.trim()) return null;
    try {
        JSON.parse(value);
        return null;
    } catch {
        return 'Not valid JSON. Enter an ordered JSON array such as [{"type":"env"},{"type":"file"}], or clear the field to use the defaults.';
    }
};
