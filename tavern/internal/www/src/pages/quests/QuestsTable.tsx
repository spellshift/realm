import { useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { QuestRowVirtualized } from "./QuestRowVirtualized";
import { VirtualizedTableHeader } from "../../components/tavern-base-ui/virtualized-table/VirtualizedTableHeader";
import { VirtualizedTable } from "../../components/tavern-base-ui/virtualized-table/VirtualizedTable";

interface QuestsTableProps {
    questIds: string[];
    hasMore?: boolean;
    onLoadMore?: () => void;
}

export const QuestsTable = ({ questIds, hasMore = false, onLoadMore }: QuestsTableProps) => {
    const navigate = useNavigate();

    const handleRowClick = useCallback((questId: string) => {
        navigate(`/tasks/${questId}`);
    }, [navigate]);

    const columns = useMemo(() => [
        "Quest details",
        "Updated",
        "Finished",
        "Output",
        "Error",
        "Creator"
    ], []);

    const gridCols = "minmax(200px,2fr) minmax(100px,1fr) minmax(80px,100px) minmax(80px,100px) minmax(80px,100px) minmax(120px,150px)";

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
        <QuestRowVirtualized
            questId={itemId}
            onRowClick={onItemClick}
            isVisible={isVisible}
        />
    ), []);

    return (
        <VirtualizedTable
            items={questIds}
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
