import { EmptyState, EmptyStateType } from "../tavern-base-ui/EmptyState";

const EmptyStateNoBeacon = () => {
    return (
        <EmptyState type={EmptyStateType.noData} label="No beacons found" details="Get started by deploying an imix agent on your target system.">
            <button
                type="button"
                className="inline-flex items-center rounded-md bg-purple-700 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-purple-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-700"
                onClick={() => window.open("https://docs.realm.pub/user-guide/getting-started#start-the-agent", '_blank')}
            >
                See imix docs
            </button>
        </EmptyState>
    );
}
export default EmptyStateNoBeacon;
