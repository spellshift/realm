import { VirtualizedTableWrapper } from "../../../components/tavern-base-ui/virtualized-table";
import { BeaconsTable } from "./BeaconsTable";
import { useBeaconIds } from "./useBeaconIds";
import { useParams } from "react-router-dom";

const BeaconTab = () => {
    const { hostId } = useParams();

    const {
        data,
        beaconIds,
        initialLoading,
        error,
        hasMore,
        loadMore,
    } = useBeaconIds(hostId || "");

    return (
        <div className="mt-2">
            <VirtualizedTableWrapper
                title="Beacons"
                totalItems={data?.beacons?.totalCount}
                loading={initialLoading}
                error={error}
                // Filters / Sorting must be expanded to support sorting on this page
                showFiltering={false}
                table={
                    <BeaconsTable
                        beaconIds={beaconIds}
                        hasMore={hasMore}
                        onLoadMore={loadMore}
                    />
                }
            />
        </div>
    );
}

export default BeaconTab;
