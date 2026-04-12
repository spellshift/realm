import { useCallback, useMemo, useState, useEffect } from "react";
import { PlugIcon, DownloadIcon } from "lucide-react";
import { Image, Tooltip } from "@chakra-ui/react";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Button from "../../../components/tavern-base-ui/button/Button";
import { VirtualizedTable } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../../components/tavern-base-ui/virtualized-table/types";
import { GET_PORTAL_DETAIL_QUERY } from "./queries";
import { PortalsQueryTopLevel, PortalNode } from "./types";
import UserImageAndName from "../../../components/UserImageAndName";
import PlaceholderUser from "../../../assets/PlaceholderUser.png";

interface PortalsTableProps {
    portalIds: string[];
    hasMore?: boolean;
    onLoadMore?: () => void;
}

export const PortalsTable = ({ portalIds, hasMore = false, onLoadMore }: PortalsTableProps) => {

    const getVariables = useCallback((id: string) => ({ id }), []);

    const extractData = useCallback((response: PortalsQueryTopLevel): PortalNode | null => {
        return response?.portals?.edges?.[0]?.node || null;
    }, []);

    const columns: VirtualizedTableColumn<PortalNode>[] = useMemo(() => [
        {
            key: "id",
            label: "Portal ID",
            width: "100px",
            render: (portal: PortalNode) => (
                <div className="flex items-center text-sm font-medium text-gray-900 truncate">
                    {portal.id}
                </div>
            ),
        },
        {
            key: "command",
            label: "Proxy Command",
            width: "minmax(300px, 1fr)",
            render: (portal: PortalNode) => {
                const protocol = window.location.protocol;
                const host = window.location.host;
                const commandStr = `./socks5 -portal=${portal.id} -upstream=${protocol}//${host}`;
                return (
                    <div className="flex items-center">
                        <code className="text-xs bg-gray-100 p-1 rounded font-mono truncate max-w-full">
                            {commandStr}
                        </code>
                    </div>
                );
            },
        },
        {
            key: "status",
            label: "Status",
            width: "100px",
            render: (portal: PortalNode) => {
                const isActive = !portal.closedAt;
                return (
                    <Badge
                        badgeStyle={{ color: isActive ? "green" : "gray" }}
                    >
                        {isActive ? "Active" : "Closed"}
                    </Badge>
                );
            },
        },
        {
            key: "owner",
            label: "Owner",
            width: "150px",
            render: (portal: PortalNode) => (
                <div className="flex items-center">
                    <UserImageAndName
                        userData={portal.owner as any}
                    />
                </div>
            ),
        },
        {
            key: "users",
            label: "Active Users",
            width: "150px",
            render: (portal: PortalNode) => {
                const isActive = !portal.closedAt;
                if (!isActive) return <div className="text-sm text-gray-500">-</div>;

                return (
                    <div className="flex flex-row">
                        {portal.activeUsers?.edges?.map((userEdge, index) => (
                            <Tooltip key={userEdge.node.id} label={userEdge.node.name}>
                                <Image
                                    key={userEdge.node.id}
                                    className={`w-6 h-6 rounded-full border border-gray-300 shadow-sm ${index !== 0 ? "-ml-2" : ""}`}
                                    src={userEdge.node.photoURL || PlaceholderUser}
                                    alt={`${userEdge.node.name}'s profile picture`}
                                    fallbackSrc={PlaceholderUser}
                                />
                            </Tooltip>
                        ))}
                    </div>
                );
            },
        },
    ], []);

    return (
        <VirtualizedTable<PortalNode, PortalsQueryTopLevel>
            items={portalIds}
            columns={columns}
            query={GET_PORTAL_DETAIL_QUERY}
            getVariables={getVariables}
            extractData={extractData}
            hasMore={hasMore}
            onLoadMore={onLoadMore}
        />
    );
};
