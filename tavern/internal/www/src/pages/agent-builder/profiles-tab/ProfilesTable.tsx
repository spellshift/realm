import { ColumnDef } from "@tanstack/react-table";
import { Pencil, Trash2 } from "lucide-react";
import { Tooltip } from "@chakra-ui/react";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Button from "../../../components/tavern-base-ui/button/Button";
import { TomeTactic } from "../../../utils/enums";
import { BuildProfile, DEFAULT_BUILD_PROFILE_NAME, TRANSPORT_TYPES } from "../types";

/**
 * Columns for the Profiles tab table. The Default build profile (seeded by
 * Tavern on startup, see ensureDefaultBuildProfile in tavern/app.go) is the
 * oldest row named Default. Later duplicates retain their actions so operators
 * can recover them.
 */
export const getProfileColumns = (
    profiles: BuildProfile[],
    onEdit: (profile: BuildProfile) => void,
    onDelete: (profile: BuildProfile) => void
): ColumnDef<BuildProfile>[] => {
    const defaultProfileId = profiles
        .filter((profile) => profile.name === DEFAULT_BUILD_PROFILE_NAME)
        .reduce<string | null>((firstId, profile) =>
            firstId === null || Number(profile.id) < Number(firstId) ? profile.id : firstId, null);

    return [
        { accessorKey: "name", header: "Name", cell: ({ row }) => <span className="text-sm font-medium text-gray-900">{row.original.name}</span> },
        { accessorKey: "description", header: "Description", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.description}</span> },
        {
            id: "transport",
            header: "Transport",
            cell: ({ row }) => {
                const transport = row.original.transports[0];
                const label = transport && (TRANSPORT_TYPES.find((type) => type.value === transport.type)?.label || transport.type);
                return <span className="text-sm text-gray-600">{label || "—"}</span>;
            },
        },
        { accessorKey: "tomeCount", header: "Tomes", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.tomeCount}</span> },
        {
            id: "tomeTypes",
            header: "Tome Types",
            cell: ({ row }) => {
                const types = Array.from(
                    new Set(
                        (row.original.tomes || [])
                            .map((t) => (t.tactic && TomeTactic[t.tactic]) || t.tactic)
                            .filter(Boolean)
                    )
                );
                if (types.length === 0) {
                    return <span className="text-sm text-gray-400">—</span>;
                }
                return (
                    <div className="flex flex-row gap-1 flex-wrap">
                        {types.map((type) => (
                            <Badge key={type} badgeStyle={{ color: "gray" }}>
                                {type}
                            </Badge>
                        ))}
                    </div>
                );
            },
        },
        {
            id: "actions",
            header: "Actions",
            cell: ({ row }) => {
                const profile = row.original;
                if (profile.id === defaultProfileId) {
                    return null;
                }
                return (
                    <div className="flex flex-row gap-2 justify-end">
                        <Tooltip label="Edit" bg="white" color="black">
                            <Button
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "xs" }}
                                leftIcon={<Pencil className="w-4 h-4" />}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onEdit(profile);
                                }}
                                aria-label="Edit"
                            />
                        </Tooltip>
                        <Tooltip label="Delete" bg="white" color="black">
                            <Button
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "xs" }}
                                leftIcon={<Trash2 className="w-4 h-4" />}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onDelete(profile);
                                }}
                                aria-label="Delete"
                            />
                        </Tooltip>
                    </div>
                );
            },
        },
    ];
};
