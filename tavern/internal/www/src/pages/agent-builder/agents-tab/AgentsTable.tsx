import { ReactElement } from "react";
import { ColumnDef } from "@tanstack/react-table";
import { Tooltip } from "@chakra-ui/react";
import { Share } from "lucide-react";
import { CheckCircleIcon, XCircleIcon } from "@heroicons/react/24/outline";
import { Ring } from "@uiball/loaders";
import Button from "../../../components/tavern-base-ui/button/Button";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import DownloadButton from "../../assets/components/DownloadButton";
import { BuildTaskRow, BuildTaskStatus, PLATFORM_LABELS, FORMAT_LABELS } from "../types";

// Status -> badge color + icon for build task rows. Queued/Claimed/Building
// (and the brief post-build "Awaiting artifact" gap) all get the same
// in-progress spinner so operators can see at a glance which rows are
// still moving; Built/Failed get a terminal check/x icon. Spinner color is
// matched to each badge's own text color (solidBadge in BadgeStyles.ts) --
// white on the solid purple "Building" badge, dark gray on the light gray
// "Queued"/"Claimed" badge -- so it isn't invisible against its background.
const STATUS_BADGE: Record<BuildTaskStatus, { color: "gray" | "purple" | "green" | "red"; icon: ReactElement }> = {
    Queued: { color: "gray", icon: <Ring size={10} lineWeight={3} speed={2} color="#111827" /> },
    Claimed: { color: "gray", icon: <Ring size={10} lineWeight={3} speed={2} color="#111827" /> },
    Building: { color: "purple", icon: <Ring size={10} lineWeight={3} speed={2} color="#FFFFFF" /> },
    "Awaiting artifact": { color: "purple", icon: <Ring size={10} lineWeight={3} speed={2} color="#FFFFFF" /> },
    Built: { color: "green", icon: <CheckCircleIcon className="w-3.5 h-3.5" /> },
    Failed: { color: "red", icon: <XCircleIcon className="w-3.5 h-3.5" /> },
};

const statusBadge = (status: BuildTaskStatus) => {
    const { color, icon } = STATUS_BADGE[status];
    return <Badge badgeStyle={{ color }} leftIcon={icon}>{status}</Badge>;
};

// Built agents tab's table -- every BuildTask is a row (see buildTaskRows
// in queries.ts), so queued/claimed/building tasks show their live status here
// instead of a separate banner. Only `Built` rows carry an asset to
// download/share; `Failed` rows surface their error in a tooltip.
// `highlightTaskId` marks the row for the task the operator just started
// (navigated here via `?task=<id>` from the wizard's "Start build") so they
// can find it in a list that may already contain many other rows.
export const buildTaskColumns = (onCreateLink: (assetId: string, assetName: string) => void, highlightTaskId?: string | null): ColumnDef<BuildTaskRow>[] => [
    {
        accessorKey: "status",
        header: "Status",
        cell: ({ row }) => (
            <div className="flex flex-row items-center gap-2">
                {statusBadge(row.original.status)}
                {row.original.id === highlightTaskId && <Badge badgeStyle={{ color: "purple" }}>New</Badge>}
            </div>
        ),
    },
    { accessorKey: "profileName", header: "Profile", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.profileName}</span> },
    { accessorKey: "platform", header: "Platform", cell: ({ row }) => <Badge badgeStyle={{ color: "gray" }}>{PLATFORM_LABELS[row.original.platform]}</Badge> },
    { accessorKey: "format", header: "Format", cell: ({ row }) => <span className="text-sm text-gray-600">{FORMAT_LABELS[row.original.format]}</span> },
    {
        accessorKey: "artifactName",
        header: "Artifact",
        cell: ({ row }) => row.original.artifactName
            ? <span className="font-mono text-xs">{row.original.artifactName}</span>
            : <span className="text-sm text-gray-400">—</span>,
    },
    { accessorKey: "artifactSize", header: "Size", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.artifactSize ?? "—"}</span> },
    { accessorKey: "timestamp", header: "Updated", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.timestamp}</span> },
    {
        id: "actions",
        header: "",
        cell: ({ row }) => {
            if (row.original.status === "Built" && row.original.assetId && row.original.assetName) {
                return (
                    <div className="flex flex-row gap-2">
                        <DownloadButton assetName={row.original.assetName} />
                        <Tooltip label="Create Link" bg="white" color="black">
                            <Button
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "xs" }}
                                leftIcon={<Share className="w-4 h-4" />}
                                onClick={(event) => {
                                    event.stopPropagation();
                                    onCreateLink(row.original.assetId!, row.original.assetName!);
                                }}
                                aria-label="Create Link"
                            />
                        </Tooltip>
                    </div>
                );
            }
            if (row.original.status === "Failed" && row.original.error) {
                return (
                    <Tooltip label={row.original.error} bg="white" color="black">
                        <span className="text-xs text-gray-500 underline cursor-help">Details</span>
                    </Tooltip>
                );
            }
            return null;
        },
    },
];
