import { DocumentNode } from "@apollo/client";
import { FieldInputParams } from "../../utils/interfacesUI";

export interface CreateQuestInitialData {
    name?: string;
    tomeId?: string | null;
    params?: FieldInputParams[];
    beacons?: string[];
    initialStep?: 0 | 1 | 2;
}

// RefetchQuery can be a DocumentNode or a string (operation name)
// Using operation names (strings) is more reliable for query matching
export type RefetchQuery = DocumentNode | string;

export interface OpenCreateQuestModalOptions {
    initialFormData?: CreateQuestInitialData;
    onComplete?: (questId: string) => void;
    navigateToQuest?: boolean;
    refetchQueries?: RefetchQuery[];
}

export interface CreateQuestModalContextType {
    isOpen: boolean;
    openModal: (options?: OpenCreateQuestModalOptions) => void;
    closeModal: () => void;
}
