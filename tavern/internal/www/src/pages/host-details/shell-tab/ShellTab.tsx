import { VirtualizedTableWrapper } from "../../../components/tavern-base-ui/virtualized-table";
import { ShellsTable } from "./ShellsTable";
import { useShellIds } from "./useShellIds";
import { useParams } from "react-router-dom";
import { CreateShellButton } from "../../../components/create-shell-button/CreateShellButton";

const ShellTab = () => {
    const { hostId } = useParams();

    const {
        data,
        shellIds,
        initialLoading,
        error,
        hasMore,
        loadMore,
    } = useShellIds(hostId || "");

    return (
        <div className="mt-2">
            <VirtualizedTableWrapper
                title="Shells"
                totalItems={data?.shells?.totalCount}
                loading={initialLoading}
                error={error}
                showFiltering={false}
                actions={<CreateShellButton hostId={hostId} />}
                table={
                    <ShellsTable
                        shellIds={shellIds}
                        hasMore={hasMore}
                        onLoadMore={loadMore}
                    />
                }
            />
        </div>
    );
}

export default ShellTab;
