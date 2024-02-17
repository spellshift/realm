import EmptyStateNoBeacon from "../../../components/empty-states/EmptyStateNoBeacon";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { useHostAcitvityData } from "../hook/useHostActivityData";
import DashboardStatistic from "./DashboardStatistic";
import GroupHostActivityTable from "./GroupHostActivityTable";

const AccessCard = ({ hosts }: { hosts: any }) => {
    const { loading, hostActivity, onlineHostCount, offlineHostCount } = useHostAcitvityData(hosts);

    if (!hosts && hosts?.length < 1) {
        <EmptyStateNoBeacon />
    }

    return (
        <div className="grid grid-cols-1 md:grid-cols-5  gap-6  bg-white rounded-lg shadow-lg p-4">
            <h3 className="text-xl col-span-1 md:col-span-5">
                Access status
            </h3>
            <div className="col-span-1 flex flex-row md:flex-col gap-4 flex-wrap">
                <DashboardStatistic label="Online hosts" value={onlineHostCount} loading={loading} />
                <DashboardStatistic label="Offline hosts" value={offlineHostCount} loading={loading} />
            </div>
            <div className="col-span-1 md:col-span-4">
                {loading ? (
                    <EmptyState type={EmptyStateType.loading} label="Formatting host data..." />
                ) : (!hostActivity || hostActivity?.length < 1) ? (
                    <EmptyState type={EmptyStateType.noData} label="Unable to format access by group tag" />
                ) : (<GroupHostActivityTable hostActivity={hostActivity} />)}
            </div>
        </div>
    );
}
export default AccessCard;
