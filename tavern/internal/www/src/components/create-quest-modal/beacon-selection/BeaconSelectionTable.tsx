import { useCallback, useMemo, useRef } from "react";
import { Checkbox } from "@chakra-ui/react";
import { VirtualizedTable } from "../../tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../tavern-base-ui/virtualized-table/types";
import BeaconTile from "../../BeaconTile";
import { BeaconNode } from "../../../utils/interfacesQuery";
import { GET_BEACON_DETAIL_QUERY } from "./queries";
import { BeaconDetailQueryResponse } from "./types";

interface BeaconSelectionTableProps {
    beaconIds: string[];
    selectedBeaconIds: string[];
    onToggle: (beaconId: string) => void;
}

export const BeaconSelectionTable = ({
    beaconIds,
    selectedBeaconIds,
    onToggle,
}: BeaconSelectionTableProps) => {
    const selectionKey = selectedBeaconIds.join(",");

    const selectedSetRef = useRef(new Set(selectedBeaconIds));
    selectedSetRef.current = new Set(selectedBeaconIds);

    const onToggleRef = useRef(onToggle);
    onToggleRef.current = onToggle;

    const getVariables = useCallback((id: string) => ({ id }), []);

    const extractData = useCallback(
        (response: BeaconDetailQueryResponse): BeaconNode | null => {
            return response?.beacons?.edges?.[0]?.node || null;
        },
        []
    );

    // Columns are stable - they use refs to access latest state
    const columns: VirtualizedTableColumn<BeaconNode>[] = useMemo(
        () => [
            {
                key: "beacon-details",
                label: "Beacons",
                width: "minmax(300px, 1fr)",
                render: (beacon) => {
                    const isSelected = selectedSetRef.current.has(beacon.id);
                    return (
                        <div className="flex flex-col min-w-0">
                            <Checkbox
                                colorScheme="purple"
                                size="lg"
                                isChecked={isSelected}
                                aria-label={`Select beacon ${beacon.name}`}
                                onChange={() => onToggle(beacon.id)}
                                >
                                <BeaconTile beacon={beacon} className="text-sm" />
                            </Checkbox>
                        </div>
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
        [selectionKey]
    );

    if (beaconIds.length === 0) {
        return (
            <div className="flex items-center justify-center py-8 text-gray-500 h-[400px]">
                No online beacons found. Try adjusting filters.
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
            estimateRowSize={80}
            height="440px"
            minHeight="200px"
            minWidth="400px"
            headerVisible={false}
        />
    );
};
