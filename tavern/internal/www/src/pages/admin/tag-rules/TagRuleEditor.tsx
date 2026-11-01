import { useMemo, useState } from "react";
import Select, { SingleValue } from "react-select";
import { HostFilterBar } from "../../../components/host-filter-bar/HostFilterBar";
import { HostFilterRow } from "../../../utils/interfacesUI";
import {
    TAG_RULE_FIELDS,
    getTagRulePredicates,
    toTagRuleConditionInputs,
    validateTagRuleRows,
} from "./constants";
import { TagRuleNode, TagRuleTagNode } from "./queries";

export interface TagRuleEditorValue {
    tagID: string;
    priority: number;
    rows: HostFilterRow[];
}

interface TagRuleEditorProps {
    initial?: TagRuleNode;
    tags: TagRuleTagNode[];
    tagsLoading: boolean;
    saving: boolean;
    error?: string;
    onSave: (value: TagRuleEditorValue) => void;
    onCancel: () => void;
}

interface TagOption {
    value: string;
    label: string;
    kind: string;
}

function rowsFromConditions(conditions: string): HostFilterRow[] {
    try {
        const parsed = JSON.parse(conditions) as Array<{
            field: HostFilterRow["field"];
            predicate: HostFilterRow["predicate"];
            values: string[];
        }>;
        return parsed.map((row, index) => ({
            id: `rule-row-${index}-${Date.now()}`,
            field: row.field,
            predicate: row.predicate,
            values: [...row.values],
        }));
    } catch {
        return [];
    }
}

export function TagRuleEditor({
    initial,
    tags,
    tagsLoading,
    saving,
    error,
    onSave,
    onCancel,
}: TagRuleEditorProps) {
    const [tagID, setTagID] = useState(initial?.tag.id ?? "");
    const [priority, setPriority] = useState<number>(initial?.priority ?? 0);
    const [rows, setRows] = useState<HostFilterRow[]>(() =>
        initial ? rowsFromConditions(initial.conditions) : []
    );
    const [touched, setTouched] = useState(false);

    const tagOptions: TagOption[] = useMemo(
        () =>
            tags.map((tag) => ({
                value: tag.id,
                label: `${tag.name} (${tag.kind})`,
                kind: tag.kind,
            })),
        [tags]
    );
    const selectedTag = tagOptions.find((option) => option.value === tagID) ?? null;

    const validationErrors = useMemo(() => validateTagRuleRows(rows), [rows]);
    const showErrors = touched && validationErrors.length > 0;
    const canSave = tagID !== "" && validationErrors.length === 0 && !saving;

    const handleSave = () => {
        setTouched(true);
        // Reuse the HostFilterBar subset: rows already carry Field + Predicate
        // + Value triples; the backend evaluates identical semantics in Go.
        const errors = validateTagRuleRows(rows);
        if (tagID === "" || errors.length > 0) {
            return;
        }
        void toTagRuleConditionInputs(rows);
        onSave({ tagID, priority, rows });
    };

    return (
        <div className="flex flex-col gap-3 rounded-md border border-gray-200 p-3" data-testid="tag-rule-editor">
            <div className="flex flex-col gap-1">
                <label className="font-medium text-gray-700">Tag</label>
                <Select<TagOption>
                    inputId="tag-rule-tag"
                    isLoading={tagsLoading}
                    options={tagOptions}
                    value={selectedTag}
                    onChange={(option: SingleValue<TagOption>) =>
                        setTagID(option?.value ?? "")
                    }
                    placeholder="Select an existing tag…"
                />
                <p className="text-xs text-gray-500">
                    Rules reference existing tags by ID; create the tag first.
                    The tag kind (group/service) fills that kind slot.
                </p>
            </div>
            <div className="flex flex-col gap-1">
                <label className="font-medium text-gray-700" htmlFor="tag-rule-priority">
                    Order (priority)
                </label>
                <input
                    id="tag-rule-priority"
                    type="number"
                    className="w-32 rounded-md border border-gray-300 px-2 py-1 text-sm"
                    value={priority}
                    onChange={(event) => setPriority(Number(event.target.value))}
                />
                <p className="text-xs text-gray-500">
                    Rules evaluate in order; the first matching rule per kind
                    slot wins. Lower values evaluate first.
                </p>
            </div>
            <HostFilterBar
                value={rows}
                onChange={setRows}
                title="Conditions"
                fields={TAG_RULE_FIELDS}
            />
            <p className="text-xs text-gray-500">
                Allowed fields: platform, primary IP, external IP, host name.
                {` Platform supports: ${getTagRulePredicates("platform").join(", ")}.`}
            </p>
            {showErrors && (
                <ul className="text-sm text-red-600" data-testid="tag-rule-errors">
                    {validationErrors.map((validationError) => (
                        <li key={`${validationError.rowId}-${validationError.message}`}>
                            {validationError.message}
                        </li>
                    ))}
                </ul>
            )}
            {touched && tagID === "" && (
                <p className="text-sm text-red-600">Select a tag.</p>
            )}
            {error && <p className="text-sm text-red-600">{error}</p>}
            <div className="flex flex-row gap-2">
                <button
                    type="button"
                    onClick={handleSave}
                    disabled={!canSave}
                    data-testid="tag-rule-save"
                    className="rounded-md bg-purple-700 px-3 py-1 text-sm text-white disabled:opacity-50"
                >
                    Save rule
                </button>
                <button
                    type="button"
                    onClick={onCancel}
                    data-testid="tag-rule-cancel"
                    className="rounded-md border border-gray-300 px-3 py-1 text-sm text-gray-700"
                >
                    Cancel
                </button>
            </div>
        </div>
    );
}

export default TagRuleEditor;
