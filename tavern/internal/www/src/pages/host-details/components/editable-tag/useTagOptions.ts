import { gql, useQuery } from "@apollo/client";
import { useMemo } from "react";
import { KindOfTag } from "../../../../utils/interfacesUI";

export const GET_TAG_OPTIONS = gql`
    query GetTagOptions($where: TagWhereInput) {
        tags(where: $where) {
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

interface TagOptionsResponse {
    tags: {
        edges: Array<{
            node: {
                id: string;
                name: string;
                kind: string;
            };
        }>;
    };
}

export function useTagOptions(kind: KindOfTag) {
    const { data, loading, error } = useQuery<TagOptionsResponse>(
        GET_TAG_OPTIONS,
        {
            variables: { where: { kind } },
            fetchPolicy: "network-only",
        }
    );

    const options = useMemo(() => {
        if (!data?.tags?.edges) {
            return [];
        }
        return data.tags.edges.map(({ node }) => ({
            id: node.id,
            name: node.name,
            value: node.id,
            label: node.name,
            kind: node.kind,
        }));
    }, [data]);

    return { options, isLoading: loading, error };
}
