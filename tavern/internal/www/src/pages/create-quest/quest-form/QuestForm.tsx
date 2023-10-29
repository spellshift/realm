import { useFormik } from "formik";
import React, { useState } from "react";
import { FormSteps } from "../../../components/form-steps";
import { useSubmitQuest } from "../../../hooks/useSubmitQuest";
import { SelectBeacons } from "../select-beacons";
import { SelectTome } from "../select-tome";

export const QuestForm = () => {
    const [currStep, setCurrStep] = useState<number>(0);
    const {submitQuest, loading, error, reset} = useSubmitQuest();

    const steps = [
        { name: 'Select a tome', description: 'Step 1', href: '#', step: 0 },
        { name: 'Select agent beacons', description: 'Step 2', href: '#', step: 1 },
    ];

    const formik = useFormik({
        initialValues: {
        name: "",
        tome: null,
        params: [],
        beacons: [],
        },
        onSubmit: (values: any) => submitQuest(values),
    } );

    function getStepView(step: number){
        switch(step) {
            case 0:
                return <SelectTome setCurrStep={setCurrStep} formik={formik} />
            case 1:
                return <SelectBeacons setCurrStep={setCurrStep} formik={formik} />
            default:
                return <div>An error has occured</div>;
        }
    }

    return (
        <form
        id='create-quest-form'
        className="py-6"
        >
            <div className="grid grid-cols-12">
                <div className=" col-span-3">
                    <FormSteps currStep={currStep} steps={steps} />
                </div>
                <div className="col-span-9">
                {getStepView(currStep)}
                </div>
            </div>
        </form>
    );
}