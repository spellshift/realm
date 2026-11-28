import { ColumnDef } from "@tanstack/react-table";
import { CircleCheck, CircleX } from "lucide-react";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import { Builder, Platform, PLATFORM_LABELS } from "../types";

const platformBadges = (platforms: Platform[]) => (
    <div className="flex flex-row gap-1 flex-wrap">
        {platforms.map((platform) => (
            <Badge key={platform} badgeStyle={{ color: "gray" }}>{PLATFORM_LABELS[platform]}</Badge>
        ))}
    </div>
);

const displayHostname = (hostname: string | null) => hostname?.trim() || "Unknown host";

const lastSeenTimestamp = (lastSeenAtRaw: string | null): number => {
    if (!lastSeenAtRaw) return -Infinity;
    const ms = Date.parse(lastSeenAtRaw);
    return Number.isNaN(ms) ? -Infinity : ms;
};

export const builderColumns: ColumnDef<Builder>[] = [
    {
        id: "hostname",
        accessorFn: (builder) => builder.hostname?.trim() ?? "",
        header: "Hostname",
        cell: ({ row }) => {
            const hostname = displayHostname(row.original.hostname);
            return (
                <div className="flex flex-row items-center gap-2 min-w-0">
                    {row.original.online ? (
                        <CircleCheck className="w-5 h-5 text-green-600 shrink-0" />
                    ) : (
                        <CircleX className="w-5 h-5 text-red-600 shrink-0" />
                    )}
                    <div className="flex flex-col min-w-0">
                        <span className="text-sm font-semibold text-gray-900 truncate" title={hostname}>
                            {hostname}
                        </span>
                        <span className="font-mono text-xs text-gray-500 truncate" title={row.original.identifier}>
                            {row.original.identifier}
                        </span>
                    </div>
                </div>
            );
        },
    },
    { accessorKey: "supportedTargets", header: "Supports", cell: ({ row }) => platformBadges(row.original.supportedTargets) },
    {
        id: "lastSeenAt",
        accessorFn: (builder) => lastSeenTimestamp(builder.lastSeenAtRaw),
        sortingFn: "basic",
        header: "Last seen",
        cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.lastSeenAt ?? "Never"}</span>,
    },
];
