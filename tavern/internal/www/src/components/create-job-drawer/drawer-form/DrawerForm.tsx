import { List } from "@chakra-ui/react";
import { useFormik } from "formik";
import React, { useState } from "react";

import { StepCustomizeParameters } from "../step-customize-parameters/StepCustomizeParameters";
import { StepJobStatus } from "../step-job-status";
import { StepSelectTargets } from "../step-select-targets/StepSelectTargets";
import { StepSelectTome } from "../step-select-tome/StepSelectTome";
import { createJobSchema } from "../../../pages/create-job/validation";

type Props = {
    onClose: () => void;
}
export const DrawerForm = (props: Props) => {
    const {onClose } = props;

    const [currStep, setCurrStep] = useState<number>(0);

    const formik = useFormik({
        initialValues: {
        tomeId: '',
        params: {
          command: "",
        },
        sessions: {},
        },
        validationSchema: createJobSchema(),
        onSubmit: values => {
            alert(JSON.stringify(values, null, 2));
        },
    });

    // const handleSetTargets = (newTargets: any) => {
    //   console.log(newTargets);
    //   formik.setFieldValue('sessions', newTargets);
    // }

    console.log(formik.values);

    return (
        <form
        id='create-job-form'
        onSubmit={(e) => formik.handleSubmit}
      >
        <List spacing={3}>
          <StepSelectTome step={0} currStep={currStep} setCurrStep={setCurrStep} formik={formik}/>
          <StepCustomizeParameters step={1} currStep={currStep} setCurrStep={setCurrStep} formik={formik}/>                 
          <StepSelectTargets step={2} currStep={currStep} setCurrStep={setCurrStep} targets={formik.values.sessions} setFieldValue={formik.setFieldValue} handleSubmit={formik.handleSubmit}/>
          <StepJobStatus step={3} currStep={currStep} setCurrStep={setCurrStep} onClose={onClose}/>
        </List>
      </form>
    );
}