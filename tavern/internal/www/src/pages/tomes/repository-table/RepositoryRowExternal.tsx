import { useMemo, useCallback } from "react";
import { Tooltip } from "@chakra-ui/react";
import { formatDistance } from "date-fns";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Button from "../../../components/tavern-base-ui/button/Button";
import TomeAccordion from "../../../components/TomeAccordion";
import UserImageAndName from "../../../components/UserImageAndName";
import { VirtualizedTableRow } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTableRow";
import { VirtualizedTableColumn } from "../../../components/tavern-base-ui/virtualized-table/types";
import { TomeNode } from "../../../utils/interfacesQuery";
import { constructTomeParams } from "../../../utils/utils";
import { GET_REPOSITORY_DETAIL_QUERY } from "../queries";
import {
    RepositoryDetailQueryResponse,
    GetRepositoryDetailQueryVariables,
    RepositoryDisplayData,
} from "../types";
import { Copy, RefreshCw } from "lucide-react";

interface RepositoryRowExternalProps {
    repositoryId: string;
    isVisible: boolean;
    isExpanded: boolean;
    onToggleExpand: (id: string) => void;
    onRefetch: (repoId: string) => void;
    isRefetching: boolean;
}

export const RepositoryRowExternal = ({
    repositoryId,
    isVisible,
    isExpanded,
    onToggleExpand,
    onRefetch,
    isRefetching,
}: RepositoryRowExternalProps) => {
    const currentDate = useMemo(() => new Date(), []);

    const getVariables = useCallback((id: string): GetRepositoryDetailQueryVariables => ({
        id,
    }), []);

    const handleRefetch = useCallback((e: React.MouseEvent, repoId: string) => {
        e.stopPropagation();
        onRefetch(repoId);
    }, [onRefetch]);

    const handleCopyPublicKey = useCallback((e: React.MouseEvent, publicKey: string | undefined) => {
        e.stopPropagation();
        if (publicKey) {
            navigator.clipboard.writeText(publicKey);
        }
    }, []);

    const columns: VirtualizedTableColumn<RepositoryDisplayData>[] = useMemo(() => [
        {
            key: 'repository',
            gridWidth: 'minmax(300px,3fr)',
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
                        </div>
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center">
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-48"></div>
                </div>
            ),
        },
        {
            key: 'uploader',
            gridWidth: 'minmax(120px,1fr)',
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
            gridWidth: 'minmax(100px,1fr)',
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
            gridWidth: 'minmax(80px,0.5fr)',
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
            gridWidth: 'minmax(100px,1fr)',
            render: (repo) => (
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
            ),
            renderSkeleton: () => (
                <div className="flex items-center gap-1">
                    <div className="h-6 w-6 bg-gray-200 rounded animate-pulse"></div>
                    <div className="h-6 w-6 bg-gray-200 rounded animate-pulse"></div>
                </div>
            ),
        },
    ], [currentDate, isRefetching, handleRefetch, handleCopyPublicKey]);

    const extractData = useCallback((response: RepositoryDetailQueryResponse): RepositoryDisplayData | null => {
        const node = response?.repositories?.edges?.[0]?.node;
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
        <VirtualizedTableRow<RepositoryDisplayData, RepositoryDetailQueryResponse>
            itemId={repositoryId}
            query={GET_REPOSITORY_DETAIL_QUERY}
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
