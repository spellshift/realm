import { ApolloError } from "@apollo/client";
import { FilterBarOption } from "../../utils/interfacesUI";

export interface FilterOptionGroup {
    label: string;
    options: FilterBarOption[];
}

export interface AssetFilterBarProps {
    userId: string;
    agentOnly: boolean;
    onChange: (updates: { userId: string; agentOnly: boolean }) => void;
    isDisabled?: boolean;
}

export interface UseAssetFilterBarResult {
    options: FilterOptionGroup[];
    isLoading: boolean;
    error: ApolloError | undefined;
}
