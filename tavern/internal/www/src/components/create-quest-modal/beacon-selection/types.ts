import { BeaconNode, QueryPageInfo, Cursor, OrderByField } from "../../../utils/interfacesQuery";
import { FilterBarOption } from "../../../utils/interfacesUI";
import { ModalQuestFormikProps } from "../types";

export interface BeaconTableProps {
    beaconIds: string[];
    selectable?: boolean;
    selectedBeaconIds?: string[];
    onToggle?: (beaconId: string) => void;
    emptyMessage?: string;
}

export interface BeaconSelectionStepProps {
    setOpen: (state: boolean) => any;
    setCurrStep: (step: number) => void;
    formik: ModalQuestFormikProps;
}

export interface BeaconSelectionRowProps {
    beacon: BeaconNode;
    isSelected: boolean;
    onToggle: (beaconId: string) => void;
}

// Beacon ID response (includes minimal data for one-per-host filtering)
export interface BeaconIdNode {
    id: string;
    principal?: string;
    transport?: string;
    host?: {
        id: string;
    };
}

export interface BeaconIdEdge {
    node: BeaconIdNode;
}

export interface BeaconIdsQueryResponse {
    totalCount: number;
    pageInfo: QueryPageInfo;
    edges: BeaconIdEdge[];
}

export interface BeaconIdsQueryTopLevel {
    beacons: BeaconIdsQueryResponse;
}

// Beacon detail response
export interface BeaconDetailQueryResponse {
    beacons: {
        edges: { node: BeaconNode }[];
    };
}

// Query variables
export interface GetBeaconIdsQueryVariables {
    where?: Record<string, unknown>;
    first?: number;
    last?: number;
    after?: Cursor;
    before?: Cursor;
    orderBy?: OrderByField[];
}

export interface GetBeaconDetailQueryVariables {
    id: string;
}

export interface UseOnlineBeaconIdsResult {
    beaconIds: string[];
    totalCount: number;
    initialLoading: boolean;
    error: Error | undefined;
    refetch: () => void;
}

export interface BeaconSelectionFilterState {
    typeFilters: FilterBarOption[];
    viewOnlySelected: boolean;
    viewOnlyOnePerHost: boolean;
}
