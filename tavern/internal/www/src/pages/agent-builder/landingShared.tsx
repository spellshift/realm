import { ColumnDef } from "@tanstack/react-table";
import { Tooltip } from "@chakra-ui/react";
import { Share } from "lucide-react";
import Button from "../../components/tavern-base-ui/button/Button";
import Badge from "../../components/tavern-base-ui/badge/Badge";
import DownloadButton from "../assets/components/DownloadButton";
import { MockBuilder, MockBuildProfile, MockBuiltAgent } from "./mockData";

const platformBadges = (platforms: string[]) => (
    <div className="flex flex-row gap-1 flex-wrap">
        {platforms.map((p) => (
            <Badge key={p} badgeStyle={{ color: "gray" }}>{p}</Badge>
        ))}
    </div>
);

export const builderColumns: ColumnDef<MockBuilder>[] = [
    { accessorKey: "identifier", header: "Identifier", cell: ({ row }) => <span className="font-mono text-xs">{row.original.identifier}</span> },
    { accessorKey: "supportedTargets", header: "Supports", cell: ({ row }) => platformBadges(row.original.supportedTargets) },
    { accessorKey: "upstream", header: "Upstream", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.upstream}</span> },
    { accessorKey: "pollInterval", header: "Poll interval", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.pollInterval}s</span> },
    {
        accessorKey: "lastSeenAt",
        header: "Last seen",
        cell: ({ row }) =>
            row.original.lastSeenAt ? (
                <Badge badgeStyle={{ color: "green" }}>{row.original.lastSeenAt}</Badge>
            ) : (
                <Badge badgeStyle={{ color: "gray" }}>Never</Badge>
            ),
    },
];

export const profileColumns: ColumnDef<MockBuildProfile>[] = [
    { accessorKey: "name", header: "Name", cell: ({ row }) => <span className="text-sm font-medium text-gray-900">{row.original.name}</span> },
    { accessorKey: "description", header: "Description", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.description}</span> },
    { accessorKey: "platforms", header: "Platforms", cell: ({ row }) => platformBadges(row.original.platforms) },
    { accessorKey: "tomeCount", header: "Tomes", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.tomeCount}</span> },
];

export const builtAgentColumns = (onCreateLink: (assetId: string, assetName: string) => void): ColumnDef<MockBuiltAgent>[] => [
    { accessorKey: "artifactName", header: "Artifact", cell: ({ row }) => <span className="font-mono text-xs">{row.original.artifactName}</span> },
    { accessorKey: "profileName", header: "Profile", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.profileName}</span> },
    { accessorKey: "platform", header: "Platform", cell: ({ row }) => <Badge badgeStyle={{ color: "gray" }}>{row.original.platform}</Badge> },
    { accessorKey: "format", header: "Format", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.format}</span> },
    { accessorKey: "artifactSize", header: "Size", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.artifactSize}</span> },
    { accessorKey: "builtAt", header: "Built", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.builtAt}</span> },
    {
        id: "actions",
        header: "",
        cell: ({ row }) => (
            <div className="flex flex-row gap-2">
                <DownloadButton assetName={row.original.assetName} />
                <Tooltip label="Create Link" bg="white" color="black">
                    <Button
                        buttonVariant="ghost"
                        buttonStyle={{ color: "gray", size: "xs" }}
                        leftIcon={<Share className="w-4 h-4" />}
                        onClick={(event) => {
                            event.stopPropagation();
                            onCreateLink(row.original.assetId, row.original.assetName);
                        }}
                        aria-label="Create Link"
                    />
                </Tooltip>
            </div>
        ),
    },
];
