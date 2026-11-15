import { useCallback } from "react";
import { ColumnDef } from "@tanstack/react-table";
import { Tooltip, useToast } from "@chakra-ui/react";
import { Share, Copy, Trash2 } from "lucide-react";
import BuildStatusIcon from "../../../components/BuildStatusIcon";
import Button from "../../../components/tavern-base-ui/button/Button";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import UserImageAndName from "../../../components/UserImageAndName";
import DownloadButton from "../../assets/components/DownloadButton";
import { BuildTaskRow, PLATFORM_LABELS } from "../types";

// Hash cell -- matches AssetsTable.tsx's truncated-hash-plus-copy pattern,
// including the "Copied to clipboard" toast on click.
const HashCell = ({ hash }: { hash: string | null }) => {
    const toast = useToast();
    const handleCopy = useCallback((event: React.MouseEvent) => {
        event.stopPropagation();
        if (!hash) return;
        navigator.clipboard.writeText(hash);
        toast({ title: "Copied to clipboard", status: "success", duration: 2000, isClosable: true });
    }, [hash, toast]);

    if (!hash) {
        return <span className="text-gray-400 font-mono text-xs">—</span>;
    }
    return (
        <div className="flex items-center gap-1 font-mono text-xs text-gray-600">
            <span>{hash.substring(0, 12)}...</span>
            <Copy className="w-3 h-3 text-gray-400 hover:text-purple-600 cursor-pointer" onClick={handleCopy} />
        </div>
    );
};

// Built agents tab's table -- every BuildTask is a row (see buildTaskRows
// in queries.ts), so queued/claimed/building tasks show their live status here
// instead of a separate banner. Only `Built` rows carry an asset to
// download/share/delete; `Failed` rows surface their error inline instead of
// a separate "Status" column. `highlightTaskId` marks the row for the task
// the operator just started (navigated here via `?task=<id>` from the
// wizard's "Start build") so they can find it in a list that may already
// contain many other rows. `onDeleteAsset` (wired to the DELETE_ASSET
// mutation in AgentsTab.tsx) is optional so the column still renders sans
// delete action if a caller doesn't supply one.
export const buildTaskColumns = (
    onCreateLink: (assetId: string, assetName: string) => void,
    highlightTaskId?: string | null,
    onDeleteAsset?: (assetId: string) => void,
): ColumnDef<BuildTaskRow>[] => [
    {
        accessorKey: "profileName",
        header: "Agent & Profile",
        size: 320,
        cell: ({ row }) => {
            const task = row.original;
            const primary = task.artifactName ?? task.profileName;
            return (
                <div className="flex flex-row items-center gap-2 min-w-0 w-full overflow-hidden">
                    <div className="shrink-0"><BuildStatusIcon status={task.status} /></div>
                    <div className="flex flex-col min-w-0 flex-1 overflow-hidden">
                        <div className="flex items-center gap-2 min-w-0 overflow-hidden">
                            <span className="block font-semibold text-gray-900 text-sm font-mono truncate" title={primary}>
                                {primary}
                            </span>
                            {task.id === highlightTaskId && <Badge badgeStyle={{ color: "purple" }}>New</Badge>}
                        </div>
                        {task.status === "Failed" ? (
                            <span
                                className="block text-xs text-red-600 font-medium truncate"
                                title={`Build failed (exit code ${task.exitCode ?? "unknown"}): ${task.error}`}
                            >
                                Build failed (exit code {task.exitCode ?? "unknown"}): {task.error}
                            </span>
                        ) : (
                            <span className="block text-xs text-gray-500 font-sans truncate" title={`Profile: ${task.profileName}`}>
                                Profile: {task.profileName}
                            </span>
                        )}
                    </div>
                </div>
            );
        },
    },
    {
        accessorKey: "platform",
        header: "Platform",
        size: 110,
        cell: ({ row }) => <Badge badgeStyle={{ color: "gray" }}>{PLATFORM_LABELS[row.original.platform]}</Badge>,
    },
    {
        accessorKey: "creator",
        header: "Creator",
        size: 160,
        cell: ({ row }) => (
            <div className="flex items-center min-w-0">
                {row.original.creator ? (
                    <UserImageAndName userData={row.original.creator} />
                ) : (
                    <span className="text-sm text-gray-400">—</span>
                )}
            </div>
        ),
    },
    {
        accessorKey: "artifactSize",
        header: "Size",
        size: 90,
        cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.artifactSize ?? "—"}</span>,
    },
    {
        accessorKey: "artifactHash",
        header: "Hash",
        size: 140,
        cell: ({ row }) => <HashCell hash={row.original.artifactHash} />,
    },
    {
        accessorKey: "timestamp",
        header: "Updated",
        size: 120,
        cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.timestamp}</span>,
    },
    {
        id: "actions",
        header: "Actions",
        size: 120,
        cell: ({ row }) => {
            const task = row.original;
            if (task.status === "Built" && task.assetId && task.assetName) {
                return (
                    <div className="flex flex-row gap-2">
                        <DownloadButton assetName={task.assetName} />
                        <Tooltip label="Create Link" bg="white" color="black">
                            <Button
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "xs" }}
                                leftIcon={<Share className="w-4 h-4" />}
                                onClick={(event) => {
                                    event.stopPropagation();
                                    onCreateLink(task.assetId!, task.assetName!);
                                }}
                                aria-label="Create Link"
                            />
                        </Tooltip>
                        {onDeleteAsset && (
                            <Tooltip label="Delete" bg="white" color="black">
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "xs" }}
                                    leftIcon={<Trash2 className="w-4 h-4" />}
                                    onClick={(event) => {
                                        event.stopPropagation();
                                        onDeleteAsset(task.assetId!);
                                    }}
                                    aria-label="Delete"
                                />
                            </Tooltip>
                        )}
                    </div>
                );
            }
            if (task.status === "Failed" && task.assetId && onDeleteAsset) {
                return (
                    <Tooltip label="Delete" bg="white" color="black">
                        <Button
                            buttonVariant="ghost"
                            buttonStyle={{ color: "gray", size: "xs" }}
                            leftIcon={<Trash2 className="w-4 h-4" />}
                            onClick={(event) => {
                                event.stopPropagation();
                                onDeleteAsset(task.assetId!);
                            }}
                            aria-label="Delete"
                        />
                    </Tooltip>
                );
            }
            // Building/Queued/Claimed/Awaiting artifact/Failed-without-asset:
            // builds can't be cancelled, so there's nothing actionable here.
            return <span className="text-sm text-gray-400">—</span>;
        },
    },
];
