import { gql, useQuery } from "@apollo/client";
import React, { useState } from "react"

import { FormRadioGroup } from "../../../components/form-radio-group"
import { FormTextArea } from "../../../components/form-text-area";
import { Tome, TomeParams } from "../../../utils/consts";
import { safelyJsonParse } from "../../../utils/utils";

type Props = {
    setCurrStep: (arg1: number) => void;
    formik: any;
}
const GET_TOMES = gql`
    query get_tomes{
        tomes {
            id
            name
            paramDefs
            description
            eldritch
        }
    }
`;

export const SelectTome = (props: Props) => {
    const { loading, error, data } = useQuery(GET_TOMES);
    const step = 0;
    const {setCurrStep, formik} = props;

    const handleSelectTome = (tome: Tome) => {
        const {params} = safelyJsonParse(tome?.paramDefs);
        formik.setFieldValue('tome', tome);
        formik.setFieldValue('params', params);
    }

    const hasAllParamsSet = formik?.values?.params.filter( (param: TomeParams) => {
        return param?.value && param?.value !== "";
    });

    const isContinueDisabled = hasAllParamsSet.length !== formik?.values?.params.length || formik?.values?.tome === null;


    const handleClickContinue = () => {
        setCurrStep(step +1);
    }

    return (
        <div className="flex flex-col gap-6">
             <FormRadioGroup data={data?.tomes || []} selected={formik?.values?.tome} setSelected={handleSelectTome} />
             {formik?.values?.params.length > 0 && formik?.values?.params.map((field: TomeParams, index: number) => {
                return (
                    <FormTextArea key={field.name} field={field} index={index} formik={formik} />
                );
             })}
             <div className="flex flex-row gap-2">
                <button
                    className="btn-primary"
                    onClick={() => handleClickContinue()}
                    disabled={isContinueDisabled}
                >
                    Continue
                </button>
             </div>
        </div>
    )
}