import { gql, useQuery } from "@apollo/client";
import React, { useState } from "react"

import { FormRadioGroup } from "../../../components/form-radio-group"
import { Tome } from "../../../utils/consts";
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
    const [selected, setSelected] = useState(formik.values.tome);

    console.log(loading);
    console.log(data);
    console.log(error);
    console.log(selected);

    const handleClickContinue = (tome: Tome) => {
        const {params} = safelyJsonParse(tome?.paramDefs);
        formik.setFieldValue('tome', tome);
        formik.setFieldValue('params', params);
        setCurrStep(step +1);
    }

    return (
        <div className="flex flex-col gap-6">
             <FormRadioGroup data={data?.tomes || []} selected={selected} setSelected={setSelected} />
             <div className="flex flex-row gap-2">
                <button
                    className="btn-primary"
                    onClick={() => handleClickContinue(selected)}
                    disabled={selected ? false : true}
                >
                    Continue
                </button>
             </div>
        </div>
    )
}