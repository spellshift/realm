import EmptyStateNoBeacon from "../../../components/empty-states/EmptyStateNoBeacon";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import BeaconTable from "./BeaconTable";
import { useHost } from "../../../context/HostContext";

const BeaconTab = () => {
    const { data: host, loading, error } = useHost();

    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Loading beacons..." />
    }
    else if (error) {
        return <EmptyState type={EmptyStateType.error} label="Error loading beacons..." />
    }
    else if (host?.beacons?.edges && host?.beacons?.edges?.length > 0) {
        return <div className="p-6"><BeaconTable beacons={host.beacons?.edges} /></div>
    }
    else {
        return <div className="p-6"><EmptyStateNoBeacon /></div>
    }
}
export default BeaconTab;
