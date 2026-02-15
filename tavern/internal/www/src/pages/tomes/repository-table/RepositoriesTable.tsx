import { useCallback, useMemo } from "react";
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
    const { importRepositoryTomes, loading: isRefetching } = useFetchRepositoryTome(undefined, true);

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
            expandable
        />
    );
};
