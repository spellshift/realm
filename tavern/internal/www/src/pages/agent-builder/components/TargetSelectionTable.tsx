import { useMemo } from "react";
import { ColumnDef, Row } from "@tanstack/react-table";
import { CheckCircleIcon } from "@heroicons/react/24/solid";

import Badge from "../../../components/tavern-base-ui/badge/Badge";
import Table from "../../../components/tavern-base-ui/table/Table";
import { CompileTarget } from "../types";

export interface TargetSelectionTableProps {
    targets: CompileTarget[];
    selectedTargetId: string | null;
    onSelectTarget: (target: CompileTarget) => void;
}

export const TargetSelectionTable = ({ targets, selectedTargetId, onSelectTarget }: TargetSelectionTableProps) => {
    const columns: ColumnDef<CompileTarget>[] = useMemo(() => [
        {
            accessorKey: "description",
            header: "Description",
            cell: ({ row }) => <span className="text-sm font-medium text-gray-900">{row.original.description}</span>,
        },
        {
            accessorKey: "formatLabel",
            header: "Format",
            cell: ({ row }) => <span className="text-sm text-gray-700">{row.original.formatLabel}</span>,
        },
        {
            accessorKey: "platformLabel",
            header: "Platform",
            cell: ({ row }) => <Badge badgeStyle={{ color: "gray" }}>{row.original.platformLabel}</Badge>,
        },
        {
            id: "selection",
            header: "",
            size: 48,
            enableSorting: false,
            cell: ({ row }) => selectedTargetId === row.original.id ? (
                <CheckCircleIcon className="w-6 h-6 text-purple-800" />
            ) : (
                <span aria-hidden="true" className="block w-5 h-5 rounded-full border-2 border-black border-opacity-10" />
            ),
        },
    ], [selectedTargetId]);

    return (
        <div className="rounded-md border border-gray-200 overflow-y-auto max-h-[280px]">
            <Table
                data={targets}
                columns={columns}
                onRowClick={(row: Row<CompileTarget>) => onSelectTarget(row.original)}
            />
        </div>
    );
};

export default TargetSelectionTable;
