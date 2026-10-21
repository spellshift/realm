import { useState } from "react";
import { CheckCircleIcon } from "@heroicons/react/24/solid";
import { ExclamationTriangleIcon } from "@heroicons/react/24/outline";
import { ChevronDown, ChevronRight } from "lucide-react";
import { Tooltip } from "@chakra-ui/react";

import Badge from "../../../components/tavern-base-ui/badge/Badge";
import CodeBlock from "../../../components/tavern-base-ui/CodeBlock";
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
    const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());

    const toggleExpand = (id: string, e: React.MouseEvent) => {
        e.stopPropagation();
        setExpandedIds((prev) => {
            const next = new Set(prev);
            if (next.has(id)) {
                next.delete(id);
            } else {
                next.add(id);
            }
            return next;
        });
    };

    const rowSize = 80;
    const maxRows = 7;
    const maxHeight = "480px";
    const minHeight = profiles.length > maxRows
        ? maxHeight
        : `${(profiles.length * rowSize) + 20}px`;

    const gridTemplateColumns = "32px minmax(180px, 1.2fr) minmax(130px, 1fr) minmax(130px, 1fr) minmax(24px, 30px)";

    return (
        <div
            className="overflow-auto border border-gray-200 rounded-lg bg-white"
            style={{ maxHeight, minHeight, width: "100%" }}
        >
            {/* Header */}
            <div
                className="bg-gray-50 sticky top-0 grid gap-2 px-4 py-3 border-b border-gray-200 z-10"
                style={{ gridTemplateColumns, minWidth: "600px" }}
            >
                <div />
                <div className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Name
                </div>
                <div className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Transport
                </div>
                <div className="text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Tomes
                </div>
                <div />
            </div>

            {/* Rows */}
            <div className="flex flex-col">
                {profiles.map((profile) => {
                    const isSelected = selectedProfileId === profile.id;
                    const isExpanded = expandedIds.has(profile.id);

                    const tacticTypes = Array.from(
                        new Set(
                            (profile.tomes || [])
                                .map((t) => (t.tactic && TomeTactic[t.tactic]) || t.tactic)
                                .filter(Boolean)
                        )
                    );

                    return (
                        <div key={profile.id} className="border-b border-gray-200 last:border-b-0">
                            <div
                                className="grid gap-2 px-4 py-4 bg-white hover:bg-gray-50 cursor-pointer focus:outline-none focus:ring-2 focus:ring-inset focus:ring-purple-500"
                                style={{ gridTemplateColumns, minWidth: "600px" }}
                                onClick={() => onSelectProfile(profile)}
                                role="button"
                                tabIndex={0}
                                aria-label={`Select profile ${profile.name}`}
                                onKeyDown={(e) => {
                                    if (e.key === "Enter" || e.key === " ") {
                                        e.preventDefault();
                                        onSelectProfile(profile);
                                    }
                                }}
                            >
                                {/* Expand Button */}
                                <button
                                    type="button"
                                    className="flex items-center justify-center cursor-pointer text-gray-500 hover:text-gray-700"
                                    onClick={(e) => toggleExpand(profile.id, e)}
                                    aria-expanded={isExpanded}
                                    aria-label={isExpanded ? "Collapse row" : "Expand row"}
                                >
                                    {isExpanded ? (
                                        <ChevronDown className="w-4 h-4 text-gray-500" />
                                    ) : (
                                        <ChevronRight className="w-4 h-4 text-gray-500" />
                                    )}
                                </button>

                                {/* Name & Warning */}
                                <div className="text-sm text-gray-700 flex items-center gap-2 min-w-0">
                                    <span className="font-medium text-gray-900 truncate">
                                        {profile.name}
                                    </span>
                                    {!profile.hasPersistenceTome && (
                                        <Tooltip label={NO_INSTALL_WARNING} hasArrow placement="right" maxW="280px">
                                            <span tabIndex={0} aria-label="No install warning" className="inline-flex text-amber-500 flex-shrink-0">
                                                <ExclamationTriangleIcon className="h-4 w-4" />
                                            </span>
                                        </Tooltip>
                                    )}
                                </div>

                                {/* Transport Column */}
                                <div className="text-sm text-gray-700 flex flex-col justify-center min-w-0">
                                    {profile.transports.length === 0 ? (
                                        <span className="text-xs text-gray-400">—</span>
                                    ) : (
                                        profile.transports.map((t, idx) => (
                                            <div key={idx} className="flex items-center gap-1.5 text-xs truncate">
                                                <span className="font-semibold uppercase text-purple-700">
                                                    {TRANSPORT_TYPES.find((tt) => tt.value === t.type)?.label || t.type}
                                                </span>
                                                {t.uri && <span className="text-gray-500 truncate">{t.uri}</span>}
                                            </div>
                                        ))
                                    )}
                                </div>

                                {/* Tomes Column */}
                                <div className="text-sm text-gray-700 flex items-center flex-wrap gap-1 min-w-0">
                                    {tacticTypes.length === 0 ? (
                                        <span className="text-xs text-gray-400">
                                            {profile.tomeCount > 0 ? `${profile.tomeCount} tomes` : "No tomes"}
                                        </span>
                                    ) : (
                                        tacticTypes.map((type) => (
                                            <Badge key={type} badgeStyle={{ color: "gray" }}>
                                                {type}
                                            </Badge>
                                        ))
                                    )}
                                </div>

                                {/* Selection Radio Indicator */}
                                <div className="flex items-center justify-center h-full text-purple-800">
                                    {isSelected ? (
                                        <CheckCircleIcon className="w-6 h-6" />
                                    ) : (
                                        <span
                                            aria-hidden="true"
                                            className="w-5 h-5 rounded-full border-2 border-black border-opacity-10"
                                        />
                                    )}
                                </div>
                            </div>

                            {/* Expandable Dropdown with Additional Profile Data */}
                            {isExpanded && (
                                <div className="bg-gray-50 border-t border-gray-200 p-4 flex flex-col gap-3 text-sm">
                                    {profile.description && (
                                        <div className="text-sm text-gray-600">
                                            <span className="font-semibold text-gray-700">Description: </span>
                                            {profile.description}
                                        </div>
                                    )}

                                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs bg-white p-3 rounded border border-gray-200">
                                        <div>
                                            <span className="font-semibold text-gray-500 uppercase tracking-wider block">
                                                Build Image
                                            </span>
                                            <span className="font-mono text-gray-800 break-all">
                                                {profile.buildImage || "Default"}
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

                                    {/* Build Script */}
                                    {profile.buildScript && (
                                        <div className="flex flex-col gap-1">
                                            <span className="text-xs font-semibold uppercase tracking-wider text-gray-500">
                                                Build Script
                                            </span>
                                            <CodeBlock code={profile.buildScript} language="bash" />
                                        </div>
                                    )}
                                </div>
                            )}
                        </div>
                    );
                })}
            </div>
        </div>
    );
};

export default ProfileSelectionTable;
