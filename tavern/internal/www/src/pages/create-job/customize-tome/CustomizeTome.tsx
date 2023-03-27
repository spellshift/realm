import React from "react";

type Props = {
    currStep: number;
    setCurrStep: (arg1: number) => void;
    formik: any;
}
export const CustomizeTome = (props: Props) => {
    const step = 1;
    const {currStep, setCurrStep, formik} = props;
    console.log(formik.values.tome);
    const params = JSON.parse(formik.values.tome.parameters);
    console.log(params);
    return (
        <div className="flex flex-col gap-6">
            <div>
                <label htmlFor="command" className="block text-base font-semibold text-gray-900">
                   Input command
                </label>
                <div className="mt-2">
                    <textarea
                        rows={4}
                        name="command"
                        id="command"
                        className="block w-full rounded-md border-0 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:py-1.5 sm:text-sm sm:leading-6"
                        defaultValue={''}
                    />
                </div>
            </div>
             <div className="flex flex-row gap-2">
                <button
                    className="inline-flex items-center rounded-md bg-gray-50 py-3 px-4 text-sm font-semibold text-purple-600 shadow-sm hover:bg-purple-100"
                    onClick={()=> setCurrStep(step -1)}
                >
                    Back
                </button>
                <button
                    className="inline-flex items-center rounded-md bg-purple-700 px-4 py-3 text-sm font-semibold text-white shadow-sm hover:bg-purple-600 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-700"
                    onClick={()=> setCurrStep(step +1)}
                >
                    Continue
                </button>
             </div>
        </div>
    )
}