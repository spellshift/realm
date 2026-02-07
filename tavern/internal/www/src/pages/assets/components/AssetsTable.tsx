import { ColumnDef } from "@tanstack/react-table";
import { format } from "date-fns";
import { AssetEdge } from "../../../utils/interfacesQuery";
import Table from "../../../components/tavern-base-ui/table/Table";
import Button from "../../../components/tavern-base-ui/button/Button";
import { ArrowDownToLine, Share, ChevronDown, ChevronRight, BookOpen, Copy, FilePlus, Info } from "lucide-react";
import { Tooltip, useToast } from "@chakra-ui/react";
import AssetAccordion from "./AssetAccordion";
import { useState, useEffect } from "react";
import UserImageAndName from "../../../components/UserImageAndName";
import { formatRelativeTime } from "../../../utils/time";

type AssetsTableProps = {
    assets: AssetEdge[];
    onCreateLink: (assetId: string, assetName: string) => void;
    onAssetUpdate: () => void;
};

const formatBytes = (bytes: number, decimals = 2) => {
    if (!+bytes) return '0 Bytes';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

const truncateAssetName = (name: string, maxLength: number = 25): string => {
    if (name.length <= maxLength) return name;

    // Check for path structure (forward or backward slashes)
    const hasPath = name.includes('/') || name.includes('\\');

    if (hasPath) {
        // Handle path truncation: prioritize keeping the filename
        const separator = name.includes('/') ? '/' : '\\';
        const parts = name.split(separator);
        const fileName = parts.pop() || "";

        // If filename itself is too long, truncate it
        if (fileName.length > maxLength) {
            return fileName.substring(0, maxLength - 3) + "...";
        }

        // Try to add parent directories until limit is reached
        let result = fileName;
        // Start from end of parts (deepest folder)
        for (let i = parts.length - 1; i >= 0; i--) {
            const part = parts[i];
            const potential = part + separator + result;
            // +3 for "..." prefix
            if (potential.length + 3 <= maxLength) {
                result = potential;
            } else {
                return "..." + separator + result;
            }
        }
        // Should not reach here if length check passed, but fallback
        return "..." + separator + result;
    }

    // Standard string truncation
    return name.substring(0, maxLength - 3) + "...";
};

const AssetsTable = ({ assets, onCreateLink, onAssetUpdate }: AssetsTableProps) => {
    const toast = useToast();
    const [windowWidth, setWindowWidth] = useState(window.innerWidth);

    useEffect(() => {
        const handleResize = () => {
            setWindowWidth(window.innerWidth);
        };
        window.addEventListener('resize', handleResize);
        return () => window.removeEventListener('resize', handleResize);
    }, []);

    const handleCopy = (text: string, e: React.MouseEvent) => {
        e.stopPropagation();
        navigator.clipboard.writeText(text);
        toast({
            title: "Copied to clipboard",
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
                if (row.original.node.links.totalCount === 0) return null;
                return (
                    <div className="flex flex-row gap-2 items-center" >
                        {row.getIsExpanded() ? <div><ChevronDown className="w-4 h-4" /></div> : <div><ChevronRight className="w-4 h-4" /></div>}
                    </div>
                );
            },
        },
        {
            id: "name",
            header: "Name",
            accessorFn: row => row.node.name,
            enableSorting: false,
            size: 250,
            cell: ({ row }) => {
                const hasTomes = row.original.node.tomes.totalCount > 0;
                const truncatedName = truncateAssetName(row.original.node.name);
                return (
                    <div className="flex items-center gap-4">
                        {hasTomes ? (
                            <Tooltip label={`${row.original.node.tomes.totalCount} associated tome(s)`} bg="white" color="black">
                                <div className="shrink-0">
                                    <BookOpen className="w-4 h-4 text-gray-500" />
                                </div>
                            </Tooltip>
                        ) : (
                            <Tooltip label="Asset is not referenced by any tomes" bg="white" color="black">
                                <div className="shrink-0">
                                    <FilePlus className="w-4 h-4 text-gray-400" />
                                </div>
                            </Tooltip>
                        )}
                        <Tooltip label={row.original.node.name} bg="white" color="black">
                            <div
                                className="cursor-pointer hover:text-purple-600 flex items-center gap-1"
                                onClick={(e) => handleCopy(row.original.node.name, e)}
                            >
                                <span>{truncatedName}</span>
                                <Copy className="w-3 h-3 text-gray-400 hover:text-purple-600" />
                            </div>
                        </Tooltip>
                    </div>
                );
            },
        },
        {
            id: "creator",
            header: "Creator",
            accessorFn: row => row.node.creator,
            enableSorting: false,
            size: 200,
            cell: ({ row }) => {
                return (
                    <div className="pr-4">
                        <UserImageAndName userData={row.original.node.creator} />
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
            header: () => (
                <div className="flex items-center gap-1">
                    Hash
                    <Tooltip label="SHA3-256 digest of the content field" bg="white" color="black">
                        <Info className="w-3 h-3 text-gray-400 cursor-help" />
                    </Tooltip>
                </div>
            ),
            accessorFn: row => row.node.hash,
            cell: ({ getValue }) => {
                const hash = getValue() as string;
                return (
                    <Tooltip label={hash} bg="white" color="black">
                        <div
                            className="font-mono text-xs cursor-pointer hover:text-purple-600 flex items-center gap-1"
                            onClick={(e) => handleCopy(hash, e)}
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
            id: "createdAt",
            header: "Created",
            accessorFn: row => row.node.createdAt,
            cell: ({ getValue }) => format(new Date(getValue() as string), "yyyy-MM-dd HH:mm"),
            enableSorting: false,
        },
        {
            id: "lastModifiedAt",
            header: "Modified",
            accessorFn: row => row.node.lastModifiedAt,
            cell: ({ getValue }) => formatRelativeTime(getValue() as string),
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
                                leftIcon={<Share className="w-4 h-4" />}
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

    const getVisibleColumns = () => {
        // Name and Actions always shown
        const visibleIds = ["expander", "name", "actions"];

        if (windowWidth >= 800) {
            visibleIds.push("creator");
        }
        if (windowWidth >= 1000) {
            visibleIds.push("size");
        }
        if (windowWidth > 1200) {
            visibleIds.push("lastModifiedAt");
        }
        if (windowWidth >= 1400) {
            visibleIds.push("hash");
        }
        if (windowWidth > 1600) {
            visibleIds.push("createdAt");
        }

        return columns.filter(col => visibleIds.includes(col.id as string));
    };

    const visibleColumns = getVisibleColumns();


    return (
        <Table
            data={assets}
            columns={visibleColumns}
            getRowCanExpand={(row) => row.original.node.links.totalCount > 0}
            onRowClick={(row, event) => {
                if (row.original.node.links.totalCount > 0) {
                    const toggle = row.getToggleExpandedHandler();
                    toggle();
                }
            }}
            renderSubComponent={({ row }) => <AssetAccordion asset={row.original.node} onUpdate={onAssetUpdate} />}
        />
    );
};

export default AssetsTable;
