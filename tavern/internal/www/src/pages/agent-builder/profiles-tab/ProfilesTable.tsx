import { ColumnDef } from "@tanstack/react-table";
import { BuildProfile } from "../types";

export const profileColumns: ColumnDef<BuildProfile>[] = [
    { accessorKey: "name", header: "Name", cell: ({ row }) => <span className="text-sm font-medium text-gray-900">{row.original.name}</span> },
    { accessorKey: "description", header: "Description", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.description}</span> },
    { accessorKey: "tomeCount", header: "Tomes", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.tomeCount}</span> },
];
