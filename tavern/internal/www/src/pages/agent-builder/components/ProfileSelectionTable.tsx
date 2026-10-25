import { useMemo } from "react";
import { ColumnDef, Row } from "@tanstack/react-table";
import { CheckCircleIcon } from "@heroicons/react/24/solid";
import { ChevronDownIcon, ChevronRightIcon, ExclamationTriangleIcon } from "@heroicons/react/24/outline";
import { Tooltip } from "@chakra-ui/react";

import Badge from "../../../components/tavern-base-ui/badge/Badge";
import CodeBlock from "../../../components/tavern-base-ui/CodeBlock";
import Table from "../../../components/tavern-base-ui/table/Table";
import { TomeTactic } from "../../../utils/enums";
import { BuildProfile, NO_INSTALL_WARNING, TRANSPORT_TYPES } from "../types";

export interface ProfileSelectionTableProps {
    profiles: BuildProfile[];
    selectedProfileId: string | null;
    onSelectProfile: (profile: BuildProfile) => void;
}

export const ProfileSelectionTable = ({ profiles, selectedProfileId, onSelectProfile }: ProfileSelectionTableProps) => {
    const columns: ColumnDef<BuildProfile>[] = useMemo(() => [
        {
            id: "expand",
            header: "",
            size: 44,
            enableSorting: false,
            cell: ({ row }) => (
                <button
                    type="button"
                    aria-label={row.getIsExpanded() ? "Collapse row" : "Expand row"}
                    onClick={(event) => { event.stopPropagation(); row.toggleExpanded(); }}
                >
                    {row.getIsExpanded() ? <ChevronDownIcon className="w-4 h-4" /> : <ChevronRightIcon className="w-4 h-4" />}
                </button>
            ),
        },
        {
            accessorKey: "name",
            header: "Name",
            cell: ({ row }) => {
                const profile = row.original;
                return (
                    <div className="flex flex-col min-w-0">
                        <div className="flex items-center gap-1.5 font-medium text-gray-900">
                            <span className="truncate">{profile.name}</span>
                            {!profile.hasPersistenceTome && (
                                <Tooltip label={NO_INSTALL_WARNING} placement="top" hasArrow>
                                    <span className="inline-flex items-center text-amber-500 cursor-help" aria-label="No persistence tome warning">
                                        <ExclamationTriangleIcon className="w-4 h-4" />
                                    </span>
                                </Tooltip>
                            )}
                        </div>
                        {profile.description && <span className="text-xs text-gray-500 truncate">{profile.description}</span>}
                    </div>
                );
            },
        },
        {
            id: "transports",
            header: "Transports",
            cell: ({ row }) => (
                <div className="flex flex-wrap gap-1">
                    {row.original.transports.length === 0 ? <span className="text-gray-400 text-xs italic">None</span> : (
                        <Badge badgeStyle={{ color: "gray" }}>
                            {TRANSPORT_TYPES.find((type) => type.value === row.original.transports[0].type)?.label || row.original.transports[0].type}
                        </Badge>
                    )}
                </div>
            ),
        },
        {
            id: "selection",
            header: "",
            size: 48,
            enableSorting: false,
            cell: ({ row }) => selectedProfileId === row.original.id ? (
                <CheckCircleIcon className="w-6 h-6 text-purple-800" />
            ) : (
                <span aria-hidden="true" className="block w-5 h-5 rounded-full border-2 border-black border-opacity-10" />
            ),
        },
    ], [selectedProfileId]);

    const renderDetails = ({ row }: { row: Row<BuildProfile> }) => {
        const profile = row.original;
        return (
            <div className="p-4 flex flex-col gap-3 text-xs">
                {profile.description && (
                    <div className="text-sm text-gray-700">{profile.description}</div>
                )}
                {/* Pipeline Info */}
                <div className="grid grid-cols-2 gap-4 bg-gray-100 p-2.5 rounded">
                    <div>
                        <span className="font-semibold text-gray-500 uppercase tracking-wider block">
                            Build Image
                        </span>
                        <span className="font-mono text-gray-800 break-all">
                            {profile.buildImage || "Default builder environment"}
                        </span>
                    </div>
                    <div>
                        <span className="font-semibold text-gray-500 uppercase tracking-wider block">
                            Artifact Path
                        </span>
                        <span className="font-mono text-gray-800 break-all">
                            {profile.artifactPath || "Standard"}
                        </span>
                    </div>
                </div>

                {/* Transport Details */}
                {profile.transports.length > 0 && (
                    <div className="flex flex-col gap-1">
                        <span className="text-xs font-semibold uppercase tracking-wider text-gray-500">
                            Transports
                        </span>
                        <div className="flex flex-col gap-1 bg-white p-2.5 rounded border border-gray-200">
                            {profile.transports.map((t, i) => (
                                <div key={i} className="flex items-center justify-between text-xs py-0.5">
                                    <div className="flex items-center gap-2">
                                        <span className="font-semibold text-purple-900">
                                            {TRANSPORT_TYPES.find((tt) => tt.value === t.type)?.label || t.type}
                                        </span>
                                        <span className="text-gray-600 font-mono">{t.uri}</span>
                                    </div>
                                    <span className="text-gray-400">{t.interval}s interval</span>
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                {/* Embedded Tomes */}
                {profile.tomes.length > 0 && (
                    <div className="flex flex-col gap-1">
                        <span className="text-xs font-semibold uppercase tracking-wider text-gray-500">
                            Embedded Tomes ({profile.tomes.length})
                        </span>
                        <div className="flex flex-col gap-1 bg-white p-2.5 rounded border border-gray-200 max-h-48 overflow-y-auto">
                            {profile.tomes.map((t, idx) => (
                                <div
                                    key={idx}
                                    className="flex items-center justify-between text-xs py-1 border-b border-gray-100 last:border-0"
                                >
                                    <span className="font-medium text-gray-800">{t.tomeName}</span>
                                    <div className="flex items-center gap-2">
                                        <Badge badgeStyle={{ color: "gray" }}>
                                            {(t.tactic && TomeTactic[t.tactic]) || t.tactic}
                                        </Badge>
                                        {t.paramSummary && (
                                            <span className="text-gray-500 text-[11px] font-mono">
                                                {t.paramSummary}
                                            </span>
                                        )}
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                {/* Build Script Preview if present */}
                {profile.buildScript && (
                    <div className="flex flex-col gap-1">
                        <span className="text-xs font-semibold uppercase tracking-wider text-gray-500">
                            Build Script
                        </span>
                        <CodeBlock code={profile.buildScript} language="bash" />
                    </div>
                )}
            </div>
        );
    };

    return (
        <div className="rounded-md border border-gray-200 overflow-y-auto max-h-[480px]">
            <Table
                data={profiles}
                columns={columns}
                onRowClick={(row: Row<BuildProfile>) => onSelectProfile(row.original)}
                getRowCanExpand={() => true}
                renderSubComponent={renderDetails}
            />
        </div>
    );
};

export default ProfileSelectionTable;
