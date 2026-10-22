import { ColumnDef } from "@tanstack/react-table";
import Badge from "../../../components/tavern-base-ui/badge/Badge";
import { TomeTactic } from "../../../utils/enums";
import { BuildProfile } from "../types";

export const profileColumns: ColumnDef<BuildProfile>[] = [
    { accessorKey: "name", header: "Name", cell: ({ row }) => <span className="text-sm font-medium text-gray-900">{row.original.name}</span> },
    { accessorKey: "description", header: "Description", cell: ({ row }) => <span className="text-sm text-gray-600">{row.original.description}</span> },
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
];
