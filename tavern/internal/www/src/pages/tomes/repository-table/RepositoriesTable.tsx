import { useCallback, useMemo } from "react";
import { Tooltip } from "@chakra-ui/react";
import { formatDistance } from "date-fns";
import { Copy, RefreshCw } from "lucide-react";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Button from "../../../components/tavern-base-ui/button/Button";
import TomeAccordion from "../../../components/TomeAccordion";
import UserImageAndName from "../../../components/UserImageAndName";
import { VirtualizedTable } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../../components/tavern-base-ui/virtualized-table/types";
import { TomeNode } from "../../../utils/interfacesQuery";
import { constructTomeParams } from "../../../utils/utils";
import { useFetchRepositoryTome } from "../hooks/useFetchRepositoryTomes";
import { GET_REPOSITORY_DETAIL_QUERY, GET_FIRST_PARTY_TOMES_QUERY } from "../queries";
import {
    RepositoryDetailQueryResponse,
    FirstPartyTomesQueryResponse,
    RepositoryDisplayData,
    FIRST_PARTY_REPO_ID,
} from "../types";

// Union type for the two possible response types
type RepositoryQueryResponse = RepositoryDetailQueryResponse | FirstPartyTomesQueryResponse;

interface RepositoriesTableProps {
    repositoryIds: string[];
}

export const RepositoriesTable = ({ repositoryIds }: RepositoriesTableProps) => {
    const { importRepositoryTomes, loading: isRefetching } = useFetchRepositoryTome(undefined, true);
    const currentDate = useMemo(() => new Date(), []);

    // Return different query based on item type
    const getQuery = useCallback((itemId: string) => {
        return itemId === FIRST_PARTY_REPO_ID
            ? GET_FIRST_PARTY_TOMES_QUERY
            : GET_REPOSITORY_DETAIL_QUERY;
    }, []);

    const getVariables = useCallback((itemId: string) => {
        return itemId === FIRST_PARTY_REPO_ID ? {} : { id: itemId };
    }, []);

    const extractData = useCallback((response: RepositoryQueryResponse, itemId: string): RepositoryDisplayData | null => {
        if (itemId === FIRST_PARTY_REPO_ID) {
            const firstPartyResponse = response as FirstPartyTomesQueryResponse;
            if (!firstPartyResponse?.tomes?.edges) return null;

            const tomes = firstPartyResponse.tomes.edges.map(edge => edge.node);
            return {
                id: FIRST_PARTY_REPO_ID,
                url: "https://github.com/spellshift/realm/tree/main/tavern/tomes",
                tomes,
                owner: null,
                isFirstParty: true,
            };
        }

        const repoResponse = response as RepositoryDetailQueryResponse;
        const node = repoResponse?.repositories?.edges?.[0]?.node;
        if (!node) return null;

        return {
            id: node.id,
            url: node.url,
            lastModifiedAt: node.lastModifiedAt,
            publicKey: node.publicKey,
            tomes: node.tomes.edges.map(edge => edge.node),
            owner: node.owner,
            isFirstParty: false,
        };
    }, []);

    const handleRefetch = useCallback((e: React.MouseEvent, repoId: string) => {
        e.stopPropagation();
        importRepositoryTomes(repoId);
    }, [importRepositoryTomes]);

    const handleCopyPublicKey = useCallback((e: React.MouseEvent, publicKey: string | undefined) => {
        e.stopPropagation();
        if (publicKey) {
            navigator.clipboard.writeText(publicKey);
        }
    }, []);

    const columns: VirtualizedTableColumn<RepositoryDisplayData>[] = useMemo(() => [
        {
            key: 'repository',
            label: 'Repository',
            width: 'minmax(300px,3fr)',
            render: (repo) => {
                const hasLink = repo.url.includes("http");
                return (
                    <div className="flex flex-row gap-2 items-center min-w-0">
                        <div className="flex flex-row gap-2 flex-wrap min-w-0">
                            {hasLink ? (
                                <a
                                    href={repo.url}
                                    target="_blank"
                                    rel="noreferrer"
                                    className="external-link truncate"
                                    onClick={(e) => e.stopPropagation()}
                                >
                                    {repo.url}
                                </a>
                            ) : (
                                <div className="break-all">{repo.url}</div>
                            )}
                            {repo.isFirstParty && (
                                <Badge badgeStyle={{ color: "purple" }}>First Party</Badge>
                            )}
                        </div>
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center gap-2">
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-48"></div>
                </div>
            ),
        },
        {
            key: 'uploader',
            label: 'Uploader',
            width: 'minmax(120px,1fr)',
            render: (repo) => (
                <div className="flex items-center min-w-0">
                    {repo.owner ? (
                        <UserImageAndName userData={repo.owner} />
                    ) : (
                        <span className="text-gray-400">-</span>
                    )}
                </div>
            ),
            renderSkeleton: () => (
                <div className="flex items-center">
                    <div className="h-8 w-8 bg-gray-200 rounded-full animate-pulse"></div>
                </div>
            ),
        },
        {
            key: 'updated',
            label: 'Updated',
            width: 'minmax(100px,1fr)',
            render: (repo) => {
                const formattedDate = repo.lastModifiedAt
                    ? formatDistance(new Date(repo.lastModifiedAt), currentDate)
                    : "-";
                return (
                    <div className="flex items-center min-w-0">
                        <span className="text-gray-600">{formattedDate}</span>
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center">
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-16"></div>
                </div>
            ),
        },
        {
            key: 'tomes',
            label: 'Tomes',
            width: 'minmax(80px,0.5fr)',
            render: (repo) => (
                <div className="flex items-center">
                    <Badge badgeStyle={{ color: "gray" }}>{repo.tomes.length}</Badge>
                </div>
            ),
            renderSkeleton: () => (
                <div className="flex items-center">
                    <div className="h-6 bg-gray-200 rounded animate-pulse w-8"></div>
                </div>
            ),
        },
        {
            key: 'actions',
            label: 'Actions',
            width: 'minmax(100px,1fr)',
            render: (repo) => {
                // First party repos don't have actions
                if (repo.isFirstParty) {
                    return <div />;
                }

                return (
                    <div className="flex items-center">
                        <div className="flex flex-row">
                            <Tooltip label="Refetch tomes">
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "xs" }}
                                    leftIcon={<RefreshCw className="w-4 h-4"/>}
                                    aria-label="Refetch tomes"
                                    disabled={isRefetching}
                                    onClick={(e) => handleRefetch(e, repo.id)}
                                />
                            </Tooltip>
                            {repo.publicKey !== "" && (
                                <Tooltip label="Copy public key">
                                    <Button
                                        buttonVariant="ghost"
                                        buttonStyle={{ color: "gray", size: "xs" }}
                                        aria-label="Copy public key"
                                        leftIcon={<Copy className="w-4 h-4" />}
                                        onClick={(e) => handleCopyPublicKey(e, repo.publicKey)}
                                    />
                                </Tooltip>
                            )}
                        </div>
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center gap-1">
                    <div className="h-6 w-6 bg-gray-200 rounded animate-pulse"></div>
                    <div className="h-6 w-6 bg-gray-200 rounded animate-pulse"></div>
                </div>
            ),
        },
    ], [currentDate, isRefetching, handleRefetch, handleCopyPublicKey]);

    const expandableConfig = useMemo(() => ({
        render: (repo: RepositoryDisplayData) => (
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
        ),
        isExpandable: (repo: RepositoryDisplayData) => repo.tomes.length > 0,
    }), []);

    return (
        <VirtualizedTable<RepositoryDisplayData, RepositoryQueryResponse>
            items={repositoryIds}
            columns={columns}
            query={getQuery}
            getVariables={getVariables}
            extractData={extractData}
            expandable={expandableConfig}
            estimateRowSize={73}
            overscan={5}
        />
    );
};
