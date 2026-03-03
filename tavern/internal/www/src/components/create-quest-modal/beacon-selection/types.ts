import { BeaconNode } from "../../../utils/interfacesQuery";
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
