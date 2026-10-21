import { useCallback, useMemo } from "react";
import { CheckCircleIcon } from "@heroicons/react/24/solid";
import { ExclamationTriangleIcon } from "@heroicons/react/24/outline";
import { Tooltip } from "@chakra-ui/react";

import Badge from "../../../components/tavern-base-ui/badge/Badge";
import CodeBlock from "../../../components/tavern-base-ui/CodeBlock";
import { VirtualizedTable } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { ExpandableConfig, VirtualizedTableColumn } from "../../../components/tavern-base-ui/virtualized-table/types";
import { TomeTactic } from "../../../utils/enums";
import { BuildProfile, NO_INSTALL_WARNING, TRANSPORT_TYPES } from "../types";

export interface ProfileSelectionTableProps {
    profiles: BuildProfile[];
    selectedProfileId: string | null;
    onSelectProfile: (profile: BuildProfile) => void;
}

export const ProfileSelectionTable = ({
    profiles,
    selectedProfileId,
    onSelectProfile,
}: ProfileSelectionTableProps) => {
    const profileMap = useMemo(() => {
        const map = new Map<string, BuildProfile>();
        for (const p of profiles) {
            map.set(p.id, p);
        }
        return map;
    }, [profiles]);

    const profileIds = useMemo(() => profiles.map((p) => p.id), [profiles]);
    const getItem = useCallback((id: string) => profileMap.get(id) ?? null, [profileMap]);

    const handleItemClick = useCallback(
        (_id: string, profile: BuildProfile | null) => {
            if (!profile) return;
            onSelectProfile(profile);
        },
        [onSelectProfile]
    );

    const expandable: ExpandableConfig<BuildProfile> = useMemo(
        () => ({
            render: (profile) => (
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
            ),
            isExpandable: () => true,
        }),
        []
    );

    const columns: VirtualizedTableColumn<BuildProfile>[] = useMemo(() => [
        {
            key: "name",
            label: "Name",
            width: "minmax(150px, 1fr)",
            render: (profile) => (
                <div className="flex flex-col min-w-0">
                    <div className="flex items-center gap-1.5 font-medium text-gray-900">
                        <span className="truncate">{profile.name}</span>
                        {!profile.hasPersistenceTome && (
                            <Tooltip label={NO_INSTALL_WARNING} placement="top" hasArrow>
                                <span
                                    className="inline-flex items-center text-amber-500 flex-shrink-0 cursor-help"
                                    aria-label="No persistence tome warning"
                                >
                                    <ExclamationTriangleIcon className="w-4 h-4" />
                                </span>
                            </Tooltip>
                        )}
                    </div>
                    {profile.description && (
                        <span className="text-xs text-gray-500 truncate">{profile.description}</span>
                    )}
                </div>
            ),
        },
        {
            key: "transports",
            label: "Transports",
            width: "minmax(120px, 1fr)",
            render: (profile) => (
                <div className="flex flex-wrap gap-1">
                    {profile.transports.length === 0 ? (
                        <span className="text-gray-400 text-xs italic">None</span>
                    ) : (
                        profile.transports.map((t, idx) => {
                            const typeLabel = TRANSPORT_TYPES.find((tt) => tt.value === t.type)?.label || t.type;
                            return (
                                <Badge key={idx} badgeStyle={{ color: "purple" }}>
                                    {typeLabel}
                                </Badge>
                            );
                        })
                    )}
                </div>
            ),
        },
        {
            key: "tomes",
            label: "Tomes",
            width: "minmax(100px, 150px)",
            render: (profile) => {
                const tacticTypes = Array.from(
                    new Set(
                        (profile.tomes || [])
                            .map((t) => (t.tactic && TomeTactic[t.tactic]) || t.tactic)
                            .filter(Boolean)
                    )
                );
                return (
                    <div className="flex flex-wrap gap-1 items-center">
                        <Badge badgeStyle={{ color: "gray" }}>
                            {profile.tomeCount} {profile.tomeCount === 1 ? "tome" : "tomes"}
                        </Badge>
                        {tacticTypes.map((tactic) => (
                            <Badge key={tactic} badgeStyle={{ color: "purple" }}>
                                {tactic}
                            </Badge>
                        ))}
                    </div>
                );
            },
        },
        {
            key: "selection",
            label: "",
            width: "minmax(24px, 30px)",
            render: (profile) => {
                if (selectedProfileId === profile.id) {
                    return (
                        <div className="flex items-center justify-center h-full text-purple-800">
                            <CheckCircleIcon className="w-6 h-6" />
                        </div>
                    );
                }
                return (
                    <div className="flex items-center justify-center h-full text-purple-800">
                        <span
                            aria-hidden="true"
                            className="w-5 h-5 rounded-full border-2 border-black border-opacity-10"
                        />
                    </div>
                );
            },
        },
    ], [selectedProfileId]);

    const rowSize = 80;
    const maxRows = 7;
    const maxHeight = "480px";
    const minHeight = profileIds.length > maxRows
        ? maxHeight
        : `${(profileIds.length * rowSize) + 20}px`;

    return (
        <VirtualizedTable<BuildProfile>
            items={profileIds}
            columns={columns}
            getItem={getItem}
            expandable={expandable}
            onItemClick={handleItemClick}
            estimateRowSize={rowSize}
            height={maxHeight}
            minHeight={minHeight}
            minWidth="300px"
            headerVisible={true}
            growWithContent={true}
        />
    );
};

export default ProfileSelectionTable;
