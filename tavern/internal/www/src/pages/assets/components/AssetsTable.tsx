import { ColumnDef } from "@tanstack/react-table";
import { format } from "date-fns";
import { AssetEdge } from "../../../utils/interfacesQuery";
import Table from "../../../components/tavern-base-ui/table/Table";
import Button from "../../../components/tavern-base-ui/button/Button";
import { ArrowDownToLine, Link, ChevronDown, ChevronRight, BookOpen, Copy } from "lucide-react";
import { Tooltip, useToast } from "@chakra-ui/react";
import AssetAccordion from "./AssetAccordion";

type AssetsTableProps = {
    assets: AssetEdge[];
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
    const toast = useToast();

    const handleCopyHash = (hash: string, e: React.MouseEvent) => {
        e.stopPropagation();
        navigator.clipboard.writeText(hash);
        toast({
            title: "Hash copied to clipboard",
            status: "success",
            duration: 2000,
            isClosable: true,
        });
    };

    const columns: ColumnDef<AssetEdge>[] = [
         {
            id: 'expander',
            header: '',
            accessorFn: row => row.node.id,
            footer: props => props.column.id,
            enableSorting: false,
            maxSize: 40,
            cell: ({ row }) => {
                return (
                    <div className="flex items-center justify-center h-full w-full" >
                        {row.getIsExpanded() ? <ChevronDown className="w-4 h-4 text-gray-500" /> : <ChevronRight className="w-4 h-4 text-gray-500" />}
                    </div>
                );
            },
        },
        {
            id: "name",
            header: "Name",
            accessorFn: row => row.node.name,
            enableSorting: false,
            cell: ({ row }) => {
                const hasTomes = row.original.node.tomes.totalCount > 0;
                return (
                    <div className="flex items-center gap-2">
                        {hasTomes && (
                            <Tooltip label={`${row.original.node.tomes.totalCount} associated tome(s)`} bg="white" color="black">
                                <BookOpen className="w-4 h-4 text-gray-500" />
                            </Tooltip>
                        )}
                        <span>{row.original.node.name}</span>
                    </div>
                );
            }
        },
        {
            id: "size",
            header: "Size",
            accessorFn: row => row.node.size,
            cell: ({ getValue }) => formatBytes(getValue() as number),
            enableSorting: false,
        },
        {
            id: "hash",
            header: "Hash",
            accessorFn: row => row.node.hash,
            cell: ({ getValue }) => {
                const hash = getValue() as string;
                return (
                    <Tooltip label="Click to copy hash" bg="white" color="black">
                        <div
                            className="font-mono text-xs cursor-pointer hover:text-purple-600 flex items-center gap-1"
                            onClick={(e) => handleCopyHash(hash, e)}
                        >
                            <span>{hash.substring(0, 12)}...</span>
                            <Copy className="w-3 h-3" />
                        </div>
                    </Tooltip>
                );
            },
            enableSorting: false,
        },
        {
            id: "links",
            header: "Links",
            accessorFn: (row) => row.node.links.totalCount,
            enableSorting: false,
        },
        {
            id: "createdAt",
            header: "Created",
            accessorFn: row => row.node.createdAt,
            cell: ({ getValue }) => format(new Date(getValue() as string), "yyyy-MM-dd HH:mm"),
            enableSorting: false,
        },
        {
            id: "actions",
            header: "Actions",
            enableSorting: false,
            cell: ({ row }) => {
                return (
                    <div className="flex flex-row gap-2">
                         <Tooltip label="Download" bg="white" color="black">
                            <a href={`/assets/download/${row.original.node.name}`} download onClick={(e) => e.stopPropagation()}>
                                <Button
                                    buttonVariant="ghost"
                                    buttonStyle={{ color: "gray", size: "xs" }}
                                    leftIcon={<ArrowDownToLine className="w-4 h-4" />}
                                    aria-label="Download"
                                />
                            </a>
                        </Tooltip>
                        <Tooltip label="Create Link" bg="white" color="black">
                            <Button
                                buttonVariant="ghost"
                                buttonStyle={{ color: "gray", size: "xs" }}
                                leftIcon={<Link className="w-4 h-4" />}
                                onClick={(e) => {
                                    e.stopPropagation(); // Prevent row expansion
                                    onCreateLink(row.original.node.id, row.original.node.name);
                                }}
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
            getRowCanExpand={() => true}
            onRowClick={(row, event) => {
                const toggle = row.getToggleExpandedHandler();
                toggle();
            }}
            renderSubComponent={({ row }) => <AssetAccordion asset={row.original.node} />}
        />
    );
};

export default AssetsTable;
