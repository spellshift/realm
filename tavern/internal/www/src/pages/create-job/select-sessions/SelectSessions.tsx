import React, { useContext, useState } from "react";
import { TagContext } from "../../../context/TagContext";
import { SelectedSessions } from "../../../utils/consts";
import { SessionView } from "./session-view";

type Props = {
    setCurrStep: (arg1: number) => void;
    formik: any;
}
export const SelectSessions = (props: Props) => {
    const {setCurrStep, formik} = props;
    const [selectedSessions, setSelectedSessions] = useState<any>({});

    const {data, isLoading, error } = useContext(TagContext);

    function isSessionSelected(){
        for (let key in selectedSessions) {
            if (selectedSessions[key] === true) {
                return true;
            } 
        }
        return false;
    }
    const hasSessionSelected = isSessionSelected();

    const handleClickContinue = (selectedSessions: SelectedSessions) => {
        const sessionToSubmit = [] as Array<string>;
        for (let key in selectedSessions) {
            if (selectedSessions[key] === true) {
               sessionToSubmit.push(key);
            } 
        }
        formik.setFieldValue('sessions', sessionToSubmit);
        formik.handleSubmit();
    }

    return (
        <div className="flex flex-col gap-6">
            <h2 className="text-xl font-semibold text-gray-900">Select agent sessions</h2>
            {isLoading || data === undefined ?
            (
                <div>
                    Loading...
                </div>
            ): (
                <SessionView sessions={data?.sessions || []} groups={data?.groupTags || []} services={data?.serviceTags || []} selectedSessions={selectedSessions} setSelectedSessions={setSelectedSessions} />
            )}
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
                        handleClickContinue(selectedSessions);
                    }}
                    disabled={!hasSessionSelected}
                    type="submit"
                >
                    Submit
                </button>
             </div>
        </div>
    );
}