import { FormikProps } from "formik";
import { FieldInputParams } from "../../utils/interfacesUI";

export interface ModalQuestFormValues {
    name: string;
    tomeId: string | null;
    params: FieldInputParams[];
    beacons: string[];
}

export type ModalQuestFormikProps = FormikProps<ModalQuestFormValues>;
