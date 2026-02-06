import { ColumnDef } from "@tanstack/react-table";
import { format } from "date-fns";
import { AssetNode } from "../../../utils/interfacesQuery";
import Table from "../../../components/tavern-base-ui/table/Table";
import Button from "../../../components/tavern-base-ui/button/Button";
import { ArrowDownTrayIcon, LinkIcon } from "@heroicons/react/24/outline";
import { Tooltip } from "@chakra-ui/react";

type AssetsTableProps = {
    assets: AssetNode[];
    onCreateLink: (assetId: string, assetName: string) => void;
};

const formatBytes = (bytes: number, decimals = 2) => {
    if (!+bytes) return '0 Bytes';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

const AssetsTable = ({ assets, onCreateLink }: AssetsTableProps) => {
    const columns: ColumnDef<AssetNode>[] = [
        {
            id: "name",
            header: "Name",
            accessorKey: "name",
        },
        {
            id: "size",
            header: "Size",
            accessorKey: "size",
            cell: ({ getValue }) => formatBytes(getValue() as number),
        },
        {
            id: "hash",
            header: "Hash",
            accessorKey: "hash",
            cell: ({ getValue }) => {
                const hash = getValue() as string;
                return (
                    <Tooltip label={hash}>
                        <span className="font-mono text-xs">{hash.substring(0, 12)}...</span>
                    </Tooltip>
                );
            },
        },
        {
            id: "links",
            header: "Links",
            accessorFn: (row) => row.links.totalCount,
        },
        {
            id: "tomes",
            header: "Tomes",
            accessorFn: (row) => row.tomes.totalCount,
        },
        {
            id: "createdAt",
            header: "Created",
            accessorKey: "createdAt",
            cell: ({ getValue }) => format(new Date(getValue() as string), "yyyy-MM-dd HH:mm"),
        },
        {
            id: "actions",
            header: "Actions",
            cell: ({ row }) => {
                return (
                    <div className="flex flex-row gap-2">
                         <Tooltip label="Download">
                            <a href={`/assets/download/${row.original.name}`} download>
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "xs" }}
                                    leftIcon={<ArrowDownTrayIcon className="w-4 h-4" />}
                                    aria-label="Download"
                                />
                            </a>
                        </Tooltip>
                        <Tooltip label="Create Link">
                            <Button
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "xs" }}
                                leftIcon={<LinkIcon className="w-4 h-4" />}
                                onClick={() => onCreateLink(row.original.id, row.original.name)}
                                aria-label="Create Link"
                            />
                        </Tooltip>
                    </div>
                );
            },
        },
    ];

    return (
        <Table
            data={assets}
            columns={columns}
            getRowCanExpand={() => false}
        />
    );
};

export default AssetsTable;
