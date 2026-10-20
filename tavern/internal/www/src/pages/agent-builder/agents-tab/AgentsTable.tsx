import { ReactElement, useCallback } from "react";
import { ColumnDef } from "@tanstack/react-table";
import { Tooltip, useToast } from "@chakra-ui/react";
import { Share, Copy, Trash2, CircleCheck, CircleX, CirclePause } from "lucide-react";
import { Ring } from "@uiball/loaders";
import Button from "../../../components/tavern-base-ui/button/Button";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import UserImageAndName from "../../../components/UserImageAndName";
import DownloadButton from "../../assets/components/DownloadButton";
import { BuildTaskRow, BuildTaskStatus, PLATFORM_LABELS } from "../types";

// Status icon for the leading cell of the "Agent & Profile" column -- mirrors
// TaskStatusField.tsx / quest task rows: a pure icon (no badge/tooltip
// wrapper) so the row's status reads at a glance alongside the agent name.
// Queued/Claimed share the same paused icon; Building and the brief
// post-build "Awaiting artifact" gap share the same in-progress spinner.
const STATUS_ICON: Record<BuildTaskStatus, ReactElement> = {
    Queued: <CirclePause className="w-5 h-5 text-gray-400 shrink-0" />,
    Claimed: <CirclePause className="w-5 h-5 text-gray-400 shrink-0" />,
    Building: <span className="shrink-0"><Ring size={16} lineWeight={2.5} speed={2} color="#7C3AED" /></span>,
    "Awaiting artifact": <span className="shrink-0"><Ring size={16} lineWeight={2.5} speed={2} color="#7C3AED" /></span>,
    Built: <CircleCheck className="w-5 h-5 text-green-600 shrink-0" />,
    Failed: <CircleX className="w-5 h-5 text-red-600 shrink-0" />,
};

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
        cell: ({ row }) => {
            const task = row.original;
            const primary = task.artifactName ?? task.profileName;
            return (
                <div className="flex flex-row items-center gap-2 min-w-[300px]">
                    {STATUS_ICON[task.status]}
                    <div className="flex flex-col">
                        <div className="flex items-center gap-2">
                            <span className="font-semibold text-gray-900 text-sm font-mono">{primary}</span>
                            {task.id === highlightTaskId && <Badge badgeStyle={{ color: "purple" }}>New</Badge>}
                        </div>
                        {task.status === "Failed" ? (
                            <span className="text-xs text-red-600 font-medium">
                                Build failed (exit code {task.exitCode ?? "unknown"}): {task.error}
                            </span>
                        ) : (
                            <span className="text-xs text-gray-500 font-sans">Profile: {task.profileName}</span>
                        )}
                    </div>
                </div>
            );
        },
    },
    { accessorKey: "platform", header: "Platform", cell: ({ row }) => <Badge badgeStyle={{ color: "gray" }}>{PLATFORM_LABELS[row.original.platform]}</Badge> },
    {
        accessorKey: "creator",
        header: "Creator",
        cell: ({ row }) => (
            <div className="flex items-center">
                {row.original.creator ? (
                    <UserImageAndName userData={row.original.creator} />
                ) : (
                    <span className="text-sm text-gray-400">—</span>
                )}
            </div>
        ),
    },
    { accessorKey: "artifactSize", header: "Size", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.artifactSize ?? "—"}</span> },
    { accessorKey: "artifactHash", header: "Hash", cell: ({ row }) => <HashCell hash={row.original.artifactHash} /> },
    { accessorKey: "timestamp", header: "Updated", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.timestamp}</span> },
    {
        id: "actions",
        header: "Actions",
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
