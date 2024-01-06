import { gql, useQuery } from "@apollo/client";
import React from "react"

import { FormTextArea } from "../../../components/form-text-area";
import { FormRadioGroup } from "../../../components/tavern-base-ui/FormRadioGroup";
import FormTextField from "../../../components/tavern-base-ui/FormTextField";
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
    const { setCurrStep, formik } = props;

    const handleSelectTome = (tome: Tome) => {
        const { params } = safelyJsonParse(tome?.paramDefs);
        formik.setFieldValue('tome', tome);
        formik.setFieldValue('params', params ? params : []);
    }

    const handleNameQuest = (name: string) => {
        formik.setFieldValue('name', name);
    }

    const hasAllParamsSet = formik?.values?.params.filter((param: TomeParams) => {
        return param?.value && param?.value !== "";
    });

    const isContinueDisabled = hasAllParamsSet.length !== formik?.values?.params.length || formik?.values?.tome === null || formik?.values?.name === "";

    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-xl font-semibold text-gray-900">Customize a quest</h2>

            <FormTextField
                htmlFor="questName"
                label="Quest name"
                placeholder={"Provide a recognizable name to this quest"}
                value={formik?.values?.name}
                onChange={(event) => handleNameQuest(event?.target?.value)}
            />
            <FormRadioGroup
                label="Select a tome"
                data={data?.tomes || []}
                selected={formik?.values?.tome}
                setSelected={handleSelectTome}
            />
            {formik?.values?.params.length > 0 && formik?.values?.params.map((field: TomeParams, index: number) => {
                return (
                    <FormTextArea
                        key={field.name}
                        field={field}
                        index={index}
                        formik={formik}
                    />
                );
            })}

            <div className="flex flex-row gap-2">
                <button
                    className="inline-flex items-center rounded-md bg-gray-50 py-3 px-4 text-sm font-semibold text-purple-600 shadow-sm hover:bg-purple-100"
                    onClick={()=> setCurrStep(0)}
                >
                     Back
                 </button>
                <button
                    className="btn-primary"
                    onClick={(event) => {
                        event.preventDefault();
                        formik.handleSubmit();
                    }}
                    disabled={isContinueDisabled}
                    type="submit"
                >
                    Submit
                </button>
            </div>
        </div>
    )
}
