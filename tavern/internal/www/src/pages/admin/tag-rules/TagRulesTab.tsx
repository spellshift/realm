import { useState } from "react";
import { useMutation, useQuery } from "@apollo/client";
import { parseTagRuleConditions } from "./queries";
import {
    CREATE_TAG_RULE,
    DELETE_TAG_RULE,
    GET_TAG_RULES,
    GET_TAG_RULE_TAG_OPTIONS,
    SWEEP_TAG_RULES,
    UPDATE_TAG_RULE,
    TagRuleNode,
    TagRuleSweepMode,
    TagRulesQueryResponse,
} from "./queries";
import { toTagRuleConditionInputs } from "./constants";
import { TagRuleEditor, TagRuleEditorValue } from "./TagRuleEditor";
import { SweepConfirmModal } from "./SweepConfirmModal";

interface TagOptionNode {
    id: string;
    name: string;
    kind: string;
}

interface TagOptionsResponse {
    tags: { edges: Array<{ node: TagOptionNode }> };
}

interface SweepResultsResponse {
    sweepTagRules: {
        mode: string;
        totalMatched: number;
        totalTagged: number;
        results: Array<{
            tagRuleID: string;
            tagID: string;
            matchedHosts: number;
            taggedHosts: number;
        }>;
    };
}

function describeConditions(conditions: string): string {
    return parseTagRuleConditions(conditions)
        .map((row) => `${row.field} ${row.predicate} ${row.values.join(", ")}`)
        .join(" AND ");
}

export function TagRulesTab() {
    const { data, loading, error, refetch } = useQuery<TagRulesQueryResponse>(GET_TAG_RULES, {
        fetchPolicy: "network-only",
    });
    const { data: tagData, loading: tagsLoading } = useQuery<TagOptionsResponse>(
        GET_TAG_RULE_TAG_OPTIONS,
        { fetchPolicy: "network-only" }
    );
    const [createRule, { loading: creating, error: createError }] = useMutation(CREATE_TAG_RULE);
    const [updateRule, { loading: updating, error: updateError }] = useMutation(UPDATE_TAG_RULE);
    const [deleteRule] = useMutation(DELETE_TAG_RULE);
    const [sweepRules, { loading: sweeping, data: sweepData, error: sweepError }] =
        useMutation<SweepResultsResponse>(SWEEP_TAG_RULES);

    const [editorOpen, setEditorOpen] = useState(false);
    const [editing, setEditing] = useState<TagRuleNode | undefined>(undefined);
    const [sweepOpen, setSweepOpen] = useState(false);

    const rules = data?.tagRules.edges.map((edge) => edge.node) ?? [];
    const tags = tagData?.tags.edges.map((edge) => edge.node) ?? [];
    const saving = creating || updating;
    const saveError = createError?.message ?? updateError?.message;

    const openCreate = () => {
        setEditing(undefined);
        setEditorOpen(true);
    };
    const openEdit = (rule: TagRuleNode) => {
        setEditing(rule);
        setEditorOpen(true);
    };
    const closeEditor = () => {
        setEditorOpen(false);
        setEditing(undefined);
    };

    const handleSave = async (value: TagRuleEditorValue) => {
        const input = {
            tagID: value.tagID,
            priority: value.priority,
            order: value.priority,
            conditions: toTagRuleConditionInputs(value.rows),
        };
        if (editing) {
            await updateRule({ variables: { tagRuleID: editing.id, input } });
        } else {
            await createRule({ variables: { input } });
        }
        closeEditor();
        await refetch();
    };

    const handleDelete = async (rule: TagRuleNode) => {
        await deleteRule({ variables: { tagRuleID: rule.id } });
        await refetch();
    };

    const handleSweepConfirm = async (mode: TagRuleSweepMode) => {
        await sweepRules({ variables: { input: { mode } } });
        setSweepOpen(false);
    };

    const sweepResult = (sweepData as SweepResultsResponse | undefined)?.sweepTagRules;

    return (
        <div className="flex flex-col gap-4" data-testid="tag-rules-tab">
            <div className="flex flex-row flex-wrap items-center gap-2">
                <button
                    type="button"
                    onClick={openCreate}
                    data-testid="tag-rule-new"
                    className="rounded-md bg-purple-700 px-3 py-1 text-sm text-white"
                >
                    New rule
                </button>
                <button
                    type="button"
                    onClick={() => setSweepOpen(true)}
                    data-testid="tag-rule-sweep"
                    className="rounded-md border border-gray-300 px-3 py-1 text-sm text-gray-700"
                >
                    Backfill…
                </button>
            </div>
            <SweepConfirmModal
                isOpen={sweepOpen}
                setOpen={setSweepOpen}
                sweeping={sweeping}
                onConfirm={(mode) => void handleSweepConfirm(mode)}
            />
            <p className="text-xs text-gray-500">
                Rules evaluate on every beacon callback and fill empty kind
                slots only. The manual sweep evaluates all rules against all
                hosts at once: fill-empty matches the callback behavior,
                overwrite replaces the current tag in each winning slot.
                Unmatched slots are always left untouched.
            </p>
            {sweepError && (
                <p className="text-sm text-red-600" data-testid="tag-rule-sweep-error">
                    {sweepError.message}
                </p>
            )}
            {sweepResult && (
                <p className="text-sm text-gray-700" data-testid="tag-rule-sweep-result">
                    {`Sweep (${sweepResult.mode}): matched ${sweepResult.totalMatched} host-rule pairs, tagged ${sweepResult.totalTagged}.`}
                </p>
            )}
            {editorOpen && (
                <TagRuleEditor
                    initial={editing}
                    tags={tags}
                    tagsLoading={tagsLoading}
                    saving={saving}
                    error={saveError}
                    onSave={(value) => void handleSave(value)}
                    onCancel={closeEditor}
                />
            )}
            {loading && <p className="text-sm text-gray-500">Loading tag rules…</p>}
            {error && <p className="text-sm text-red-600">{error.message}</p>}
            {!loading && !error && (
                <ul className="flex flex-col gap-2" data-testid="tag-rule-list">
                    {rules.map((rule) => (
                        <li
                            key={rule.id}
                            className="flex flex-col gap-1 rounded-md border border-gray-200 p-3"
                            data-testid={`tag-rule-${rule.id}`}
                        >
                            <div className="flex flex-row items-center gap-2 text-sm">
                                <span className="font-medium">{rule.tag.name}</span>
                                <span className="text-gray-500">({rule.tag.kind})</span>
                                <span className="text-gray-500">order {rule.order}</span>
                            </div>
                            <p className="text-sm text-gray-700">
                                {describeConditions(rule.conditions)}
                            </p>
                            <div className="flex flex-row gap-2">
                                <button
                                    type="button"
                                    onClick={() => openEdit(rule)}
                                    data-testid={`tag-rule-edit-${rule.id}`}
                                    className="rounded-md border border-gray-300 px-2 py-1 text-xs text-gray-700"
                                >
                                    Edit
                                </button>
                                <button
                                    type="button"
                                    onClick={() => void handleDelete(rule)}
                                    data-testid={`tag-rule-delete-${rule.id}`}
                                    className="rounded-md border border-gray-300 px-2 py-1 text-xs text-gray-700"
                                >
                                    Delete
                                </button>
                            </div>
                        </li>
                    ))}
                    {rules.length === 0 && (
                        <li className="text-sm text-gray-500">No tag rules yet.</li>
                    )}
                </ul>
            )}
        </div>
    );
}

export default TagRulesTab;
