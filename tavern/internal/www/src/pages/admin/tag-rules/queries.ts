import { gql } from "@apollo/client";
import { HostFilterRow } from "../../../utils/interfacesUI";

export interface TagRuleTagNode {
    id: string;
    name: string;
    kind: string;
}

export interface TagRuleNode {
    id: string;
    priority: number;
    order: number;
    conditions: string;
    tag: TagRuleTagNode;
}

export interface TagRuleEdge {
    node: TagRuleNode;
}

export interface TagRulesQueryResponse {
    tagRules: {
        totalCount: number;
        edges: TagRuleEdge[];
    };
}

export interface TagRuleRows {
    rows: HostFilterRow[];
}

// Parses the stored conditions JSON back into editor rows.
export function parseTagRuleConditions(
    conditions: string
): HostFilterRow[] {
    try {
        const parsed = JSON.parse(conditions) as Array<{
            field: string;
            predicate: string;
            values: string[];
        }>;
        return parsed.map((row, index) => ({
            id: `rule-row-${index}`,
            field: row.field as HostFilterRow["field"],
            predicate: row.predicate as HostFilterRow["predicate"],
            values: [...row.values],
        }));
    } catch {
        return [];
    }
}

export const GET_TAG_RULES = gql`
    query GetTagRules {
        tagRules(orderBy: [{ field: PRIORITY, direction: ASC }]) {
            totalCount
            edges {
                node {
                    id
                    priority
                    order
                    conditions
                    tag {
                        id
                        name
                        kind
                    }
                }
            }
        }
    }
`;

export const GET_TAG_RULE_TAG_OPTIONS = gql`
    query GetTagRuleTagOptions {
        tags {
            edges {
                node {
                    id
                    name
                    kind
                }
            }
        }
    }
`;

export const CREATE_TAG_RULE = gql`
    mutation CreateTagRule($input: CreateTagRuleInput!) {
        createTagRule(input: $input) {
            id
            priority
            order
            conditions
            tag {
                id
                name
                kind
            }
        }
    }
`;

export const UPDATE_TAG_RULE = gql`
    mutation UpdateTagRule($tagRuleID: ID!, $input: UpdateTagRuleInput!) {
        updateTagRule(tagRuleID: $tagRuleID, input: $input) {
            id
            priority
            order
            conditions
            tag {
                id
                name
                kind
            }
        }
    }
`;

export const DELETE_TAG_RULE = gql`
    mutation DeleteTagRule($tagRuleID: ID!) {
        deleteTagRule(tagRuleID: $tagRuleID)
    }
`;

export type TagRuleSweepMode = "fill-empty" | "overwrite";

export const SWEEP_TAG_RULES = gql`
    mutation SweepTagRules($input: SweepTagRulesInput!) {
        sweepTagRules(input: $input) {
            mode
            totalMatched
            totalTagged
            results {
                tagRuleID
                tagID
                matchedHosts
                taggedHosts
            }
        }
    }
`;
