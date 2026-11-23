import Breadcrumbs from "../../components/Breadcrumbs";
import { CreateQuestDropdown } from "../../components/create-quest-modal/CreateQuestDropdown";
import { VirtualizedTableWrapper } from "../../components/tavern-base-ui/virtualized-table";
import { HostsTable } from "./HostsTable";
import { useHostIds } from "./useHostIds";
import { PageNavItem } from "../../utils/enums";

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
            <div className="flex flex-row justify-between w-full items-center">
                <Breadcrumbs
                    pages={[{
                        label: "Hosts",
                        link: "/hosts"
                    }]}
                />
                <div>
                    <CreateQuestDropdown />
                </div>
            </div>
            <VirtualizedTableWrapper
                title="Hosts"
                totalItems={data?.hosts?.totalCount}
                loading={initialLoading}
                error={error}
                sortType={PageNavItem.hosts}
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
