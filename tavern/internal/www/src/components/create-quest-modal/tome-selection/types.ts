import { TomeNode } from "../../../utils/interfacesQuery";
import { FieldInputParams } from "../../../utils/interfacesUI";
import { ModalQuestFormikProps } from "../types";

export interface TomeSelectionStepProps {
    setCurrStep: (step: number) => void;
    formik: ModalQuestFormikProps;
}

export interface TomeIdNode {
    id: string;
    name: string;
    paramDefs: string | null;
}

export interface TomeIdEdge {
    node: TomeIdNode;
}

export interface TomeIdsQueryResponse {
    edges: TomeIdEdge[];
}

export interface TomeIdsQueryTopLevel {
    tomes: TomeIdsQueryResponse;
}

export interface TomeDetailQueryResponse {
    tomes: {
        edges: { node: TomeNode }[];
    };
}

export interface GetTomeIdsQueryVariables {
    where?: Record<string, unknown>;
}

export interface GetTomeDetailQueryVariables {
    id: string;
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
