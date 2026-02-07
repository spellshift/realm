import { useCallback, useMemo, useState } from "react";
import { VirtualizedTableHeader } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTableHeader";
import { VirtualizedTable } from "../../../components/tavern-base-ui/virtualized-table/VirtualizedTable";
import { RepositoryRowFirstParty } from "./RepositoryRowFirstParty";
import { RepositoryRowExternal } from "./RepositoryRowExternal";
import { useFetchRepositoryTome } from "../hooks/useFetchRepositoryTomes";
import { FIRST_PARTY_REPO_ID } from "../types";

interface RepositoriesTableProps {
    repositoryIds: string[];
}

export const RepositoriesTable = ({ repositoryIds }: RepositoriesTableProps) => {
    const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());
    const { importRepositoryTomes, loading: isRefetching } = useFetchRepositoryTome(undefined, true);

    const handleToggleExpand = useCallback((repoId: string) => {
        setExpandedItems(prev => {
            const next = new Set(prev);
            if (next.has(repoId)) {
                next.delete(repoId);
            } else {
                next.add(repoId);
            }
            return next;
        });
    }, []);

    const handleRefetch = useCallback((repoId: string) => {
        importRepositoryTomes(repoId);
    }, [importRepositoryTomes]);

    const columns = useMemo(() => [
        "",
        "Repository",
        "Uploader",
        "Updated",
        "Tomes",
        "Actions"
    ], []);

    const gridCols = "32px minmax(300px,3fr) minmax(120px,1fr) minmax(100px,1fr) minmax(80px,0.5fr) minmax(100px,1fr)";

    const renderHeader = useCallback(() => (
        <VirtualizedTableHeader
            columns={columns}
            gridCols={gridCols}
            minWidth="800px"
        />
    ), [columns]);

    const renderRow = useCallback(({
        itemId,
        isVisible,
        isExpanded,
        onToggleExpand,
    }: {
        itemId: string;
        isVisible: boolean;
        onItemClick: (id: string) => void;
        isExpanded: boolean;
        onToggleExpand: (id: string) => void;
    }) => {
        if (itemId === FIRST_PARTY_REPO_ID) {
            return (
                <RepositoryRowFirstParty
                    isVisible={isVisible}
                    isExpanded={isExpanded}
                    onToggleExpand={onToggleExpand}
                />
            );
        }

        return (
            <RepositoryRowExternal
                repositoryId={itemId}
                isVisible={isVisible}
                isExpanded={isExpanded}
                onToggleExpand={onToggleExpand}
                onRefetch={handleRefetch}
                isRefetching={isRefetching}
            />
        );
    }, [handleRefetch, isRefetching]);

    return (
        <VirtualizedTable
            items={repositoryIds}
            renderRow={renderRow}
            renderHeader={renderHeader}
            estimateRowSize={73}
            overscan={5}
            expandedItems={expandedItems}
            onToggleExpand={handleToggleExpand}
        />
    );
};
