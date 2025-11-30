import { FormikProps } from "formik";
import { TomeNode } from "../../utils/interfacesQuery";
import { FieldInputParams } from "../../utils/interfacesUI";

export interface QuestFormValues {
    name: string;
    tome: TomeNode | null;
    params: FieldInputParams[];
    beacons: string[];
}

export type QuestFormikProps = FormikProps<QuestFormValues>;

export interface LocationStateData {
    step?: number;
    name?: string;
    tome?: TomeNode | null;
    params?: FieldInputParams[];
    beacons?: string[];
}
