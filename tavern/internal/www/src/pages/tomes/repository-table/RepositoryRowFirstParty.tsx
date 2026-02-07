import { useMemo, useCallback } from "react";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import TomeAccordion from "../../../components/TomeAccordion";
import { VirtualizedTableRow } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTableRow";
import { VirtualizedTableColumn } from "../../../components/tavern-base-ui/virtualized-table/types";
import { TomeNode } from "../../../utils/interfacesQuery";
import { constructTomeParams } from "../../../utils/utils";
import { GET_FIRST_PARTY_TOMES_QUERY } from "../queries";
import {
    FirstPartyTomesQueryResponse,
    FIRST_PARTY_REPO_ID,
    RepositoryDisplayData,
} from "../types";

interface RepositoryRowFirstPartyProps {
    isVisible: boolean;
    isExpanded: boolean;
    onToggleExpand: (id: string) => void;
}

export const RepositoryRowFirstParty = ({
    isVisible,
    isExpanded,
    onToggleExpand,
}: RepositoryRowFirstPartyProps) => {
    const getVariables = useCallback(() => ({}), []);

    const columns: VirtualizedTableColumn<RepositoryDisplayData>[] = useMemo(() => [
        {
            key: 'repository',
            gridWidth: 'minmax(300px,3fr)',
            render: (repo) => (
                <div className="flex flex-row gap-2 items-center min-w-0">
                    <div className="flex flex-row gap-2 flex-wrap min-w-0">
                        <a
                            href={repo.url}
                            target="_blank"
                            rel="noreferrer"
                            className="external-link truncate"
                            onClick={(e) => e.stopPropagation()}
                        >
                            {repo.url}
                        </a>
                        <Badge badgeStyle={{ color: "purple" }}>First Party</Badge>
                    </div>
                </div>
            ),
            renderSkeleton: () => (
                <div className="flex items-center gap-2">
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-48"></div>
                    <div className="h-5 bg-gray-200 rounded animate-pulse w-20"></div>
                </div>
            ),
        },
        {
            key: 'uploader',
            gridWidth: 'minmax(120px,1fr)',
            render: () => <span className="text-gray-400">-</span>,
            renderSkeleton: () => (
                <div className="h-8 w-8 bg-gray-200 rounded-full animate-pulse"></div>
            ),
        },
        {
            key: 'updated',
            gridWidth: 'minmax(100px,1fr)',
            render: () => <span className="text-gray-600">-</span>,
            renderSkeleton: () => (
                <div className="h-4 bg-gray-200 rounded animate-pulse w-16"></div>
            ),
        },
        {
            key: 'tomes',
            gridWidth: 'minmax(80px,0.5fr)',
            render: (repo) => (
                <Badge badgeStyle={{ color: "gray" }}>{repo.tomes.length}</Badge>
            ),
            renderSkeleton: () => (
                <div className="h-6 bg-gray-200 rounded animate-pulse w-8"></div>
            ),
        },
        {
            key: 'actions',
            gridWidth: 'minmax(100px,1fr)',
            render: () => <div />,
            renderSkeleton: () => <div />,
        },
    ], []);

    const extractData = useCallback((response: FirstPartyTomesQueryResponse): RepositoryDisplayData | null => {
        if (!response?.tomes?.edges) return null;

        const tomes = response.tomes.edges.map(edge => edge.node);
        return {
            id: FIRST_PARTY_REPO_ID,
            url: "https://github.com/spellshift/realm/tree/main/tavern/tomes",
            tomes,
            owner: null,
            isFirstParty: true,
        };
    }, []);

    const isExpandable = useCallback((repo: RepositoryDisplayData) => {
        return repo.tomes.length > 0;
    }, []);

    const renderExpandedContent = useCallback((repo: RepositoryDisplayData) => (
        <div className="px-8 py-4">
            {repo.tomes.map((tome: TomeNode) => {
                const params = constructTomeParams("[]", tome.paramDefs);
                return (
                    <div key={tome.id}>
                        <TomeAccordion tome={tome} params={params} showParamValues={false} />
                    </div>
                );
            })}
        </div>
    ), []);

    return (
        <VirtualizedTableRow<RepositoryDisplayData, FirstPartyTomesQueryResponse>
            itemId={FIRST_PARTY_REPO_ID}
            query={GET_FIRST_PARTY_TOMES_QUERY}
            getVariables={getVariables}
            columns={columns}
            extractData={extractData}
            isVisible={isVisible}
            isExpanded={isExpanded}
            onToggleExpand={onToggleExpand}
            renderExpandedContent={renderExpandedContent}
            isExpandable={isExpandable}
        />
    );
};
