import { useCallback, useMemo, useRef } from "react";
import { Checkbox } from "@chakra-ui/react";
import { VirtualizedTable } from "../../tavern-base-ui/virtualized-table/VirtualizedTable";
import { VirtualizedTableColumn } from "../../tavern-base-ui/virtualized-table/types";
import BeaconTile from "../../BeaconTile";
import { BeaconNode, BeaconDetailQueryResponse } from "../../../utils/interfacesQuery";
import { GET_BEACON_DETAIL_QUERY } from "../../../utils/queries";
import { BeaconTableProps } from "./types";

export const BeaconTable = ({
    beaconIds,
    selectable = false,
    selectedBeaconIds = [],
    onToggle,
    emptyMessage = "No beacons found",
}: BeaconTableProps) => {
    const rowSize = 80;
    const maxRows = 5;
    const maxHeight = maxRows * rowSize;
    const calculatedHeight = beaconIds.length > maxRows
        ? `${maxHeight}px`
        : `${beaconIds.length * rowSize}px`;

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

    const columns: VirtualizedTableColumn<BeaconNode>[] = useMemo(
        () => [
            {
                key: "beacon-details",
                label: "Beacons",
                width: "minmax(300px, 1fr)",
                render: (beacon) => {
                    if (!selectable) {
                        return <BeaconTile beacon={beacon} className="text-sm" />;
                    }
                    const isSelected = selectedSetRef.current.has(beacon.id);
                    return (
                        <div className="flex flex-col min-w-0">
                            <Checkbox
                                colorScheme="purple"
                                size="lg"
                                isChecked={isSelected}
                                aria-label={`Select beacon ${beacon.name}`}
                                onChange={() => onToggleRef.current?.(beacon.id)}
                            >
                                <BeaconTile beacon={beacon} className="text-sm" />
                            </Checkbox>
                        </div>
                    );
                },
                renderSkeleton: () => (
                    <div className="flex flex-col min-w-0 space-y-2">
                        <div className="h-4 bg-gray-200 rounded animate-pulse w-3/4"></div>
                        <div className="h-3 bg-gray-200 rounded animate-pulse w-1/2"></div>
                    </div>
                ),
            },
        ],
        [selectable]
    );

    if (beaconIds.length === 0) {
        return (
            <div className="flex items-center justify-center py-8 text-gray-500 h-[400px]">
                {emptyMessage}
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
            height={calculatedHeight}
            minHeight={calculatedHeight}
            minWidth="200px"
            headerVisible={false}
        />
    );
};

export default BeaconTable;
