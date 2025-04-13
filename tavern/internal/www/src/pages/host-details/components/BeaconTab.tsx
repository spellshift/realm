import EmptyStateNoBeacon from "../../../components/empty-states/EmptyStateNoBeacon";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import BeaconTable from "./BeaconTable";
import { useContext } from "react";
import { HostContext } from "../../../context/HostContext";

const BeaconTab = () => {
    const { data: host, loading, error } = useContext(HostContext);

    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Loading beacons..." />
    }
    else if (error) {
        return <EmptyState type={EmptyStateType.error} label="Error loading beacons..." />
    }
    else {
        return (
            <div className="py-4">
                {host?.beacons && host?.beacons?.length > 0 ? (
                    <BeaconTable beacons={host.beacons} />
                )
                    : (
                        <EmptyStateNoBeacon />
                    )}
            </div>
        )
    }
}
export default BeaconTab;
