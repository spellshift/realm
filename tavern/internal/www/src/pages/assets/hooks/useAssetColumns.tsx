import { ColumnDef } from "@tanstack/react-table";
import { format } from "date-fns";
import { AssetEdge } from "../../../utils/interfacesQuery";
import Button from "../../../components/tavern-base-ui/button/Button";
import { ArrowDownToLine, Share, ChevronDown, ChevronRight, BookOpen, Copy, FilePlus, Info } from "lucide-react";
import { Tooltip, useToast } from "@chakra-ui/react";
import UserImageAndName from "../../../components/UserImageAndName";
import moment from "moment";
import { formatBytes } from "../../../utils/formatters";
import { truncateAssetName } from "../utils";
import { useMemo } from "react";

interface UseAssetColumnsProps {
    onCreateLink: (assetId: string, assetName: string) => void;
}

export const useAssetColumns = ({ onCreateLink }: UseAssetColumnsProps) => {
    const toast = useToast();

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

    const columns: ColumnDef<AssetEdge>[] = useMemo(() => [
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
                            className="font-mono text-sm cursor-pointer hover:text-purple-600 flex items-center gap-1"
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
            cell: ({ getValue }) => moment(getValue() as string).fromNow(),
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
    ], [onCreateLink, toast]); // Added toast to dependency array

    return columns;
};
