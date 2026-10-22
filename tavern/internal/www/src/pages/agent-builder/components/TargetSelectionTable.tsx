import { useCallback, useMemo } from "react";
import { CheckCircleIcon } from "@heroicons/react/24/solid";

import Badge from "../../../components/tavern-base-ui/badge/Badge";
import { VirtualizedTable } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../../components/tavern-base-ui/virtualized-table/types";
import { CompileTarget } from "../types";

export interface TargetSelectionTableProps {
    targets: CompileTarget[];
    selectedTargetId: string | null;
    onSelectTarget: (target: CompileTarget) => void;
}

export const TargetSelectionTable = ({
    targets,
    selectedTargetId,
    onSelectTarget,
}: TargetSelectionTableProps) => {
    const targetMap = useMemo(() => {
        const map = new Map<string, CompileTarget>();
        for (const t of targets) {
            map.set(t.id, t);
        }
        return map;
    }, [targets]);

    const targetIds = useMemo(() => targets.map((t) => t.id), [targets]);
    const getItem = useCallback((id: string) => targetMap.get(id) ?? null, [targetMap]);

    const handleItemClick = useCallback(
        (_id: string, target: CompileTarget | null) => {
            if (!target) return;
            onSelectTarget(target);
        },
        [onSelectTarget]
    );

    const columns: VirtualizedTableColumn<CompileTarget>[] = useMemo(() => [
        {
            key: "platform",
            label: "Platform",
            width: "minmax(120px, 160px)",
            render: (target) => (
                <Badge badgeStyle={{ color: "gray" }}>{target.platformLabel}</Badge>
            ),
        },
        {
            key: "format",
            label: "Format",
            width: "minmax(160px, 220px)",
            render: (target) => (
                <span className="text-sm font-medium text-gray-900">{target.formatLabel}</span>
            ),
        },
        {
            key: "description",
            label: "Description",
            width: "minmax(200px, 1fr)",
            render: (target) => (
                <span className="text-xs text-gray-500 truncate">{target.description}</span>
            ),
        },
        {
            key: "selection",
            label: "",
            width: "minmax(24px, 30px)",
            render: (target) => {
                if (selectedTargetId === target.id) {
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
    ], [selectedTargetId]);

    const rowSize = 48;
    const maxRows = 7;
    const maxHeight = "280px";
    const minHeight = targetIds.length > maxRows
        ? maxHeight
        : `${(targetIds.length * rowSize) + 20}px`;

    return (
        <VirtualizedTable<CompileTarget>
            items={targetIds}
            columns={columns}
            getItem={getItem}
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

export default TargetSelectionTable;
