import Breadcrumbs from "../../components/Breadcrumbs";
import { VirtualizedTableWrapper } from "../../components/tavern-base-ui/virtualized-table";
import { HostsTable } from "./HostsTable";
import { useHostIds } from "./useHostIds";

const Hosts = () => {
    const {
        data,
        hostIds,
        initialLoading,
        error,
        hasMore,
        loadMore,
    } = useHostIds();

    return (
        <>
            <Breadcrumbs
                pages={[{
                    label: "Hosts",
                    link: "/hosts"
                }]}
            />
            <VirtualizedTableWrapper
                title="Hosts"
                totalItems={data?.hosts?.totalCount}
                loading={initialLoading}
                error={error}
                table={
                    <HostsTable
                        hostIds={hostIds}
                        hasMore={hasMore}
                        onLoadMore={loadMore}
                    />
                }
            />
        </>
    );
}

export default Hosts;
