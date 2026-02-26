import { useCallback, useMemo } from "react";
import { ArrowDownToLine } from "lucide-react";
import Tooltip from "../../../components/tavern-base-ui/Tooltip";
import Button from "../../../components/tavern-base-ui/button/Button";
import { VirtualizedTable } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../../components/tavern-base-ui/virtualized-table/types";
import { GET_FILE_DETAIL_QUERY } from "./queries";
import { FileDetailQueryResponse, FileNode } from "./types";
import { format } from "date-fns";
import { formatBytes } from "../../../utils/utils";

interface FilesTableProps {
    hostId: string;
    fileIds: string[];
}

export const FilesTable = ({ hostId, fileIds }: FilesTableProps) => {
    const getVariables = useCallback((id: string) => ({ hostId, fileId: id }), [hostId]);

    const extractData = useCallback((response: FileDetailQueryResponse): FileNode | null => {
        return response?.hosts?.edges?.[0]?.node?.files?.edges?.[0]?.node || null;
    }, []);

    const columns: VirtualizedTableColumn<FileNode>[] = useMemo(() => [
        {
            key: 'path',
            label: 'Path',
            width: 'minmax(200px,2fr)',
            render: (file: FileNode) => (
                <Tooltip label={file.path} isDisabled={!file.path}>
                        {file.path}
                </Tooltip>
            ),
        },
        {
            key: 'owner',
            label: 'Owner',
            width: 'minmax(120px,1fr)',
            render: (file: FileNode) => file.owner || "-",
        },
        {
            key: 'group',
            label: 'Group',
            width: 'minmax(60px,0.75fr)',
            render: (file: FileNode) => file.group || "-",
        },
        {
            key: 'permissions',
            label: 'Permissions',
            width: 'minmax(60px,0.75fr)',
            render: (file: FileNode) => file.permissions || "-",
        },
        {
            key: 'size',
            label: 'Size',
            width: 'minmax(60px,0.50fr)',
            render: (file: FileNode) => (
                <Tooltip label={`${file.size} bytes`}>
                        {formatBytes(file.size)} 
                </Tooltip>
            ),
        },
        {
            key: 'hash',
            label: 'Hash',
            width: 'minmax(100px,1fr)',
            render: (file: FileNode) => (
                <Tooltip label={file.hash || ''} isDisabled={!file.hash}>
                    {file.hash ? file.hash.substring(0, 12) + '...' : '-'}
                </Tooltip>
            ),
        },
        {
            key: 'lastModifiedAt',
            label: 'Last modified',
            width: 'minmax(120px,1fr)',
            render: (file: FileNode) => format(new Date(file.lastModifiedAt), "yyyy-MM-dd HH:mm"),
        },
        {
            key: 'actions',
            label: '',
            width: 'minmax(30px,0.3fr)',
            render: (file: FileNode) => (
                <Tooltip label="Download">
                    <a href={`/cdn/hostfiles/${file.id}`} download onClick={(e) => e.stopPropagation()}>
                        <Button
                            buttonVariant="ghost"
                            buttonStyle={{ color: "gray", size: "xs" }}
                            leftIcon={<ArrowDownToLine className="w-4 h-4" />}
                            aria-label="Download"
                        />
                    </a>
                </Tooltip>
            ),
        },
    ], []);

    return (
        <VirtualizedTable<FileNode, FileDetailQueryResponse>
            items={fileIds}
            columns={columns}
            query={GET_FILE_DETAIL_QUERY}
            getVariables={getVariables}
            extractData={extractData}
            height="60vh"
        />
    );
};
