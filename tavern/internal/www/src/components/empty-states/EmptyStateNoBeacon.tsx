import { EmptyState, EmptyStateType } from "../tavern-base-ui/EmptyState";
import Button from "../tavern-base-ui/button/Button";

/** This component takes no props. */
export type EmptyStateNoBeaconProps = Record<string, never>;

export const EmptyStateNoBeacon = () => {
    return (
        <EmptyState type={EmptyStateType.noData} label="No beacons found" details="Get started by deploying an imix agent on your target system.">
            <Button
                type="button"
                onClick={() => window.open("https://docs.realm.pub/user-guide/getting-started#start-the-agent", '_blank')}
            >
                See imix docs
            </Button>
        </EmptyState>
    );
}
export default EmptyStateNoBeacon;
