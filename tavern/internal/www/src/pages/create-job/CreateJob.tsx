import { ArrowLeftIcon } from "@heroicons/react/24/outline";
import { useFormik } from "formik";
import React, { useRef, useState } from "react";
import { Link } from "react-router-dom";
import { FormSteps } from "../../components/form-steps";
import { PageWrapper } from "../../components/page-wrapper";
import { SelectSessions } from "./select-sessions";
import { SelectTome } from "./select-tome";
import { createJobSchema } from "./validation";

export const CreateJob = () => {
    const [currStep, setCurrStep] = useState<number>(0);

    const steps = [
        { name: 'Select a tome', description: 'Step 1', href: '#', step: 0 },
        { name: 'Select agent sessions', description: 'Step 2', href: '#', step: 1 },
        { name: 'Job finalized', description: 'Done', href: '#', step: 2 },
    ];

    const formik = useFormik({
        initialValues: {
        tome: null,
        params: [],
        sessions: [],
        },
        onSubmit: values => {
            console.log("here");
            console.log(values);
            alert(JSON.stringify(values, null, 2));
        },
    });

    function getStepView(step: number){
        switch(step) {
            case 0:
                return <SelectTome setCurrStep={setCurrStep} formik={formik} />
            case 1:
                return <SelectSessions setCurrStep={setCurrStep} formik={formik} />
            case 2:
                return <div>{formik.values.sessions.join(",")}</div>
            default:
                return <div>{step}</div>;
        }
    }

    return (
        <PageWrapper>
            <div className="border-b border-gray-200 pb-6 sm:flex sm:items-center sm:justify-between">
                <h3 className="text-2xl font-semibold leading-6 text-gray-900">Create new job</h3>
                <div className="mt-3 sm:mt-0 sm:ml-4">            
                    <Link to="/jobs">
                        <button
                            type="button"
                            className="inline-flex items-center gap-2 rounded-md bg-white px-6 py-4 text-sm font-semibold shadow-sm ring-gray-300 hover:bg-gray-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-gray-300"
                        >
                            <ArrowLeftIcon className="-ml-0.5 h-5 w-5" aria-hidden="true" />
                            Back
                        </button>
                    </Link>
                </div>
            </div>
            <form
                id='create-job-form'
                className="py-6"
                //onSubmit={(e) => formik.handleSubmit}
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
        </PageWrapper>
    );
}