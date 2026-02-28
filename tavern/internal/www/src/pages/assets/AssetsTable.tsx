import { useCallback, useMemo } from "react";
import { format } from "date-fns";
import { ArrowDownToLine, Share, BookOpen, Copy, FilePlus } from "lucide-react";
import { Tooltip, useToast } from "@chakra-ui/react";
import moment from "moment";
import { VirtualizedTable } from "../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../components/tavern-base-ui/virtualized-table/types";
import { AssetNode } from "../../utils/interfacesQuery";
import Button from "../../components/tavern-base-ui/button/Button";
import UserImageAndName from "../../components/UserImageAndName";
import { truncateAssetName } from "./utils";
import { GET_ASSET_DETAIL_QUERY } from "./queries";
import { AssetDetailQueryResponse } from "./types";
import AssetAccordion from "./components/AssetAccordion";
import { formatBytes } from "../../utils/utils";

interface AssetsTableProps {
    assetIds: string[];
    hasMore?: boolean;
    onLoadMore?: () => void;
    onCreateLink: (assetId: string, assetName: string) => void;
    onAssetUpdate: () => void;
}

export const AssetsTable = ({ assetIds, hasMore = false, onLoadMore, onCreateLink, onAssetUpdate }: AssetsTableProps) => {
    const toast = useToast();

    const handleCopy = useCallback((text: string, e: React.MouseEvent) => {
        e.stopPropagation();
        navigator.clipboard.writeText(text);
        toast({
            title: "Copied to clipboard",
            status: "success",
            duration: 2000,
            isClosable: true,
        });
    }, [toast]);

    const getVariables = useCallback((id: string) => ({ id }), []);

    const extractData = useCallback((response: AssetDetailQueryResponse): AssetNode | null => {
        return response?.assets?.edges?.[0]?.node || null;
    }, []);

    const columns: VirtualizedTableColumn<AssetNode>[] = useMemo(() => [
        {
            key: 'name',
            label: 'Name',
            width: 'minmax(250px,3fr)',
            render: (asset) => {
                const hasTomes = asset.tomes.totalCount > 0;
                const truncatedName = truncateAssetName(asset.name);
                return (
                    <div className="flex items-center gap-4">
                        {hasTomes ? (
                            <Tooltip label={`${asset.tomes.totalCount} associated tome(s)`} bg="white" color="black">
                                <div className="shrink-0">
                                    <BookOpen className="w-4 h-4 text-gray-500" />
                                </div>
                            </Tooltip>
                        ) : (
                            <Tooltip label="Asset is not referenced by any tomes" bg="white" color="black">
                                <div className="shrink-0">
                                    <FilePlus className="w-4 h-4 text-gray-400" />
                                </div>
                            </Tooltip>
                        )}
                        <Tooltip label={asset.name} bg="white" color="black">
                            <div
                                className="cursor-pointer hover:text-purple-600 flex items-center gap-1"
                                onClick={(e) => handleCopy(asset.name, e)}
                            >
                                <span>{truncatedName}</span>
                                <Copy className="w-3 h-3 text-gray-400 hover:text-purple-600" />
                            </div>
                        </Tooltip>
                    </div>
                );
            },
            renderSkeleton: () => (
                <div className="flex items-center gap-4">
                    <div className="h-4 w-4 bg-gray-200 rounded animate-pulse"></div>
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4"></div>
                </div>
            ),
        },
        {
            key: 'creator',
            label: 'Creator',
            width: 'minmax(200px,2fr)',
            render: (asset) => (
                <div className="pr-4">
                    <UserImageAndName userData={asset.creator} />
                </div>
            ),
            renderSkeleton: () => (
                <div className="flex items-center gap-2">
                    <div className="h-8 w-8 bg-gray-200 rounded-full animate-pulse"></div>
                    <div className="h-4 bg-gray-200 rounded animate-pulse w-24"></div>
                </div>
            ),
        },
        {
            key: 'size',
            label: 'Size',
            width: 'minmax(100px,1fr)',
            render: (asset) => <span>{formatBytes(asset.size)}</span>,
            renderSkeleton: () => (
                <div className="h-4 bg-gray-200 rounded animate-pulse w-16"></div>
            ),
        },
        {
            key: 'hash',
            label: 'Hash',
            width: 'minmax(150px,2fr)',
            render: (asset) => {
                const hash = asset.hash;
                return (
                    <Tooltip label={hash} bg="white" color="black">
                        <div
                            className="font-mono text-sm cursor-pointer hover:text-purple-600 flex items-center gap-1"
                            onClick={(e) => handleCopy(hash, e)}
                        >
                            <span>{hash.substring(0, 12)}...</span>
                            <Copy className="w-3 h-3" />
                        </div>
                    </Tooltip>
                );
            },
            renderSkeleton: () => (
                <div className="h-4 bg-gray-200 rounded animate-pulse w-24 font-mono"></div>
            ),
        },
        {
            key: 'lastModifiedAt',
            label: 'Modified',
            width: 'minmax(120px,1fr)',
            render: (asset) => (
                <span>{moment(asset.lastModifiedAt).fromNow()}</span>
            ),
            renderSkeleton: () => (
                <div className="h-4 bg-gray-200 rounded animate-pulse w-20"></div>
            ),
        },
        {
            key: 'createdAt',
            label: 'Created',
            width: 'minmax(150px,1fr)',
            render: (asset) => (
                <span>{format(new Date(asset.createdAt), "yyyy-MM-dd HH:mm")}</span>
            ),
            renderSkeleton: () => (
                <div className="h-4 bg-gray-200 rounded animate-pulse w-32"></div>
            ),
        },
        {
            key: 'actions',
            label: 'Actions',
            width: 'minmax(100px,1fr)',
            render: (asset) => (
                <div className="flex flex-row gap-2">
                    <Tooltip label="Download" bg="white" color="black">
                        <a href={`/assets/download/${asset.name}`} download onClick={(e) => e.stopPropagation()}>
                            <Button
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "xs" }}
                                leftIcon={<ArrowDownToLine className="w-4 h-4" />}
                                aria-label="Download"
                            />
                        </a>
                    </Tooltip>
                    <Tooltip label="Create Link" bg="white" color="black">
                        <Button
                            buttonVariant="ghost"
                            buttonStyle={{ color: "gray", size: "xs" }}
                            leftIcon={<Share className="w-4 h-4" />}
                            onClick={(e) => {
                                e.stopPropagation();
                                onCreateLink(asset.id, asset.name);
                            }}
                            aria-label="Create Link"
                        />
                    </Tooltip>
                </div>
            ),
            renderSkeleton: () => (
                <div className="flex flex-row gap-2">
                    <div className="h-8 w-8 bg-gray-200 rounded animate-pulse"></div>
                    <div className="h-8 w-8 bg-gray-200 rounded animate-pulse"></div>
                </div>
            ),
        },
    ], [handleCopy, onCreateLink]);

    return (
        <VirtualizedTable<AssetNode, AssetDetailQueryResponse>
            items={assetIds}
            columns={columns}
            query={GET_ASSET_DETAIL_QUERY}
            getVariables={getVariables}
            extractData={extractData}
            hasMore={hasMore}
            onLoadMore={onLoadMore}
            expandable={{
                render: (asset) => <AssetAccordion asset={asset} onUpdate={onAssetUpdate} />,
                isExpandable: (asset) => asset.links.totalCount > 0,
            }}
        />
    );
};

export default AssetsTable;
