import { TomeNode } from "../../../utils/interfacesQuery";
import { FieldInputParams } from "../../../utils/interfacesUI";
import { ModalQuestFormikProps } from "../types";
import { Filters } from "../../../context/FilterContext";

export interface TomeTableProps {
    tomeIds: string[];
    selectable?: boolean;
    selectedTomeId?: string | null;
    onSelectTome?: (tome: TomeNode) => void;
}

export interface TomeTableWrapperProps {
    tomeIds?: string[];
    selectable?: boolean;
    selectedTomeId?: string | null;
    onSelectTome?: (tome: TomeNode) => void;
    showFilters?: boolean;
    emptyMessage?: string;
    initialFilters?: Partial<Filters>;
}

export interface TomeSelectionStepProps {
    setCurrStep: (step: number) => void;
    formik: ModalQuestFormikProps;
    initialFilters?: Partial<Filters>;
}

export interface UseTomesResult {
    tomeIds: string[];
    initialLoading: boolean;
    error: Error | undefined;
    refetch: () => void;
}

export interface TomeCardProps {
    tome: TomeNode;
    isSelected: boolean;
    onSelect: (tome: TomeNode) => void;
}

export interface ParamFieldProps {
    field: FieldInputParams;
    index: number;
    value: string;
    error?: string;
    touched?: boolean;
    onChange: (index: number, value: string) => void;
    onBlur: (index: number) => void;
}
