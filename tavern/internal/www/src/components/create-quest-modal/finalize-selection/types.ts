import { ModalQuestFormikProps } from "../types";

export interface FinalizeSelectionProps {
    setCurrStep: (step: number) => void;
    formik: ModalQuestFormikProps;
    loading?: boolean;
}
