import { useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { HostRowVirtualized } from "./HostRowVirtualized";
import { VirtualizedTableHeader } from "../../components/tavern-base-ui/virtualized-table/VirtualizedTableHeader";
import { VirtualizedTable } from "../../components/tavern-base-ui/virtualized-table/VirtualizedTable";

interface HostsTableProps {
    hostIds: string[];
    hasMore?: boolean;
    onLoadMore?: () => void;
}

export const HostsTable = ({ hostIds, hasMore = false, onLoadMore }: HostsTableProps) => {
    const navigate = useNavigate();

    const handleRowClick = useCallback((hostId: string) => {
        navigate(`/hosts/${hostId}`);
    }, [navigate]);

    const columns = useMemo(() => [
        "Host details",
        "Online beacons",
        "Beacon principals",
        "Last callback"
    ], []);

    const gridCols = "minmax(250px,3fr) minmax(120px,1fr) minmax(150px,2fr) minmax(120px,1fr)";

    const renderHeader = useCallback(() => (
        <VirtualizedTableHeader
            columns={columns}
            gridCols={gridCols}
            minWidth="800px"
        />
    ), [columns]);

    const renderRow = useCallback(({ itemId, isVisible, onItemClick }: {
        itemId: string;
        isVisible: boolean;
        onItemClick: (id: string) => void;
    }) => (
        <HostRowVirtualized
            hostId={itemId}
            onRowClick={onItemClick}
            isVisible={isVisible}
        />
    ), []);

    return (
        <VirtualizedTable
            items={hostIds}
            renderRow={renderRow}
            renderHeader={renderHeader}
            onItemClick={handleRowClick}
            hasMore={hasMore}
            onLoadMore={onLoadMore}
            estimateRowSize={73}
            overscan={5}
            height="calc(100vh - 180px)"
            minHeight="400px"
        />
    );
};
