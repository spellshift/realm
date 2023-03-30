import React from "react";
import { FormTextArea } from "../../../components/form-text-area";
import { TomeParams } from "../../../utils/consts";

type Props = {
    currStep: number;
    setCurrStep: (arg1: number) => void;
    formik: any;
}
export const CustomizeTome = (props: Props) => {
    const step = 1;
    const {currStep, setCurrStep, formik} = props;

    const isDisabled = formik?.values?.params?.filter((field: TomeParams)=> {
        return field?.value && field?.value !== ""
    }).length < formik?.values?.params?.length;

    return (
        <div className="flex flex-col gap-6">
            {formik.values.params.map((field: TomeParams, index: number) => {
                return (
                    <FormTextArea key={field.name} field={field} index={index} formik={formik} />
                );
            })}
             <div className="flex flex-row gap-2">
                <button
                    className="inline-flex items-center rounded-md bg-gray-50 py-3 px-4 text-sm font-semibold text-purple-600 shadow-sm hover:bg-purple-100"
                    onClick={()=> setCurrStep(step -1)}
                >
                    Back
                </button>
                <button
                    className="btn-primary"
                    onClick={()=> setCurrStep(step +1)}
                    disabled={isDisabled}
                >
                    Continue
                </button>
             </div>
        </div>
    )
}