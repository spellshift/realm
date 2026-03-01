import { useCallback, useMemo } from "react";
import { VirtualizedTable } from "../../tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../tavern-base-ui/virtualized-table/types";
import BeaconTile from "../../BeaconTile";
import { BeaconNode } from "../../../utils/interfacesQuery";
import { BeaconDetailQueryResponse } from "../beacon-selection";
import { GET_BEACON_DETAIL_QUERY } from "../beacon-selection/queries";


export const BeaconTable = ({beaconIds}: {beaconIds: string[]}) => {
    const getVariables = useCallback((id: string) => ({ id }), []);

    const rowSize = 80;
    const height = beaconIds.length > 5 ? "400px" : (`${beaconIds.length} * ${rowSize}` + "px");

    const extractData = useCallback(
        (response: BeaconDetailQueryResponse): BeaconNode | null => {
            return response?.beacons?.edges?.[0]?.node || null;
        },
        []
    );

    const columns: VirtualizedTableColumn<BeaconNode>[] = useMemo(
        () => [
            {
                key: "beacon-details",
                label: "Beacons",
                width: "minmax(300px, 1fr)",
                render: (beacon) => {
                    return (
                        <BeaconTile beacon={beacon} className="text-sm" />
                    )
                },
                renderSkeleton: () => (
                    <div className="flex flex-col min-w-0 space-y-2">
                        <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4"></div>
                        <div className="h-3 bg-gray-200 rounded animate-pulse w-1/2"></div>
                    </div>
                ),
            },
        ],
        []
    );

    if (beaconIds.length === 0) {
        return (
            <div className="flex items-center justify-center py-8 text-gray-500 h-[400px]">
                Unable to find beacons
            </div>
        );
    }

    return (
        <VirtualizedTable<BeaconNode, BeaconDetailQueryResponse>
            items={beaconIds}
            columns={columns}
            query={GET_BEACON_DETAIL_QUERY}
            getVariables={getVariables}
            extractData={extractData}
            estimateRowSize={rowSize}
            height={height}
            minHeight="83px"
            minWidth="200px"
            headerVisible={false}
        />
    );
};
