import { ApolloError } from "@apollo/client";
import EmptyStateNoBeacon from "../../../components/empty-states/EmptyStateNoBeacon";
import { EmptyState, EmptyStateType } from "../../../components/tavern-base-ui/EmptyState";
import { HostType } from "../../../utils/consts";
import BeaconTable from "./BeaconTable";

type Props = {
    loading: boolean;
    error: ApolloError | undefined;
    host: HostType | null;
}
const BeaconTableWrapper = (props: Props) => {
    const { loading, error, host } = props;

    if (loading) {
        return <EmptyState type={EmptyStateType.loading} label="Loading beacons..." />
    }
    else if (error) {
        return <EmptyState type={EmptyStateType.error} label="Error loading beacons..." />
    }
    else {
        return (
            <div>
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
export default BeaconTableWrapper;
