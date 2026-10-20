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

export const builderColumns: ColumnDef<Builder>[] = [
    {
        accessorKey: "identifier",
        header: "Identifier",
        cell: ({ row }) => (
            <div className="flex flex-row items-center gap-2">
                {row.original.online ? (
                    <CircleCheck className="w-5 h-5 text-green-600 shrink-0" />
                ) : (
                    <CircleX className="w-5 h-5 text-red-600 shrink-0" />
                )}
                <span className="font-mono text-xs font-semibold text-gray-900">{row.original.identifier}</span>
            </div>
        ),
    },
    { accessorKey: "supportedTargets", header: "Supports", cell: ({ row }) => platformBadges(row.original.supportedTargets) },
    {
        accessorKey: "lastSeenAt",
        header: "Last seen",
        cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.lastSeenAt ?? "Never"}</span>,
    },
];
