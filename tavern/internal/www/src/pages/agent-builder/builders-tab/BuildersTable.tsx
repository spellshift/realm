import { ColumnDef } from "@tanstack/react-table";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import { Builder, Platform, PLATFORM_LABELS } from "../types";

const platformBadges = (platforms: Platform[]) => (
    <div className="flex flex-row gap-1 flex-wrap">
        {platforms.map((platform) => (
            <Badge key={platform} badgeStyle={{ color: "gray" }}>{PLATFORM_LABELS[platform]}</Badge>
        ))}
    </div>
);

export const builderColumns: ColumnDef<Builder>[] = [
    { accessorKey: "identifier", header: "Identifier", cell: ({ row }) => <span className="font-mono text-xs">{row.original.identifier}</span> },
    { accessorKey: "supportedTargets", header: "Supports", cell: ({ row }) => platformBadges(row.original.supportedTargets) },
    { accessorKey: "upstream", header: "Upstream", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.upstream}</span> },
    { accessorKey: "pollInterval", header: "Poll interval", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.pollInterval}s</span> },
    {
        accessorKey: "lastSeenAt",
        header: "Last seen",
        cell: ({ row }) =>
            row.original.lastSeenAt ? (
                <Badge badgeStyle={{ color: row.original.online ? "green" : "gray" }}>{row.original.lastSeenAt}</Badge>
            ) : (
                <Badge badgeStyle={{ color: "gray" }}>Never</Badge>
            ),
    },
    {
        id: "online",
        header: "Status",
        cell: ({ row }) => (
            <Badge badgeStyle={{ color: row.original.online ? "green" : "red" }}>
                {row.original.online ? "Online" : "Offline"}
            </Badge>
        ),
    },
];
