import { gql, useQuery } from "@apollo/client";
import React, { useState } from "react"

import { FormRadioGroup } from "../../../components/form-radio-group"
import { Tome } from "../../../utils/consts";

type Props = {
    setCurrStep: (arg1: number) => void;
    formik: any;
}
const GET_TOMES = gql`
    query get_tomes{
        tomes {
            id
            name
            parameters
            description
            eldritch
        }
    }
`;

export const SelectTome = (props: Props) => {
    const { loading, error, data } = useQuery(GET_TOMES);
    const step = 0;
    const {setCurrStep, formik} = props;
    const [selected, setSelected] = useState(formik.values.tome);

    console.log(loading);
    console.log(data);
    console.log(error);
    console.log(selected);

    const handleClickContinue = (tome: Tome) => {
        formik.setFieldValue('tome', tome);
        setCurrStep(step +1);
    }

    return (
        <div className="flex flex-col gap-6">
             <FormRadioGroup data={data?.tomes || []} selected={selected} setSelected={setSelected} />
             <div className="flex flex-row gap-2">
                <button
                    className="inline-flex items-center rounded-md bg-purple-700 px-4 py-3 text-sm font-semibold text-white shadow-sm enabled:hover:bg-purple-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-700 disabled:opacity-50 disabled:cursor-not-allowed"
                    onClick={() => handleClickContinue(selected)}
                    disabled={selected ? false : true}
                >
                    Continue
                </button>
             </div>
        </div>
    )
}