import { FieldInputParams } from "../../utils/interfacesUI";

export interface CreateQuestInitialData {
    name?: string;
    tomeId?: string | null;
    params?: FieldInputParams[];
    beacons?: string[];
}

export interface OpenCreateQuestModalOptions {
    initialFormData?: CreateQuestInitialData;
    onComplete?: (questId: string) => void;
}

export interface CreateQuestModalContextType {
    isOpen: boolean;
    openModal: (options?: OpenCreateQuestModalOptions) => void;
    closeModal: () => void;
}
